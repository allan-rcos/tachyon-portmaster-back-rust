//! Ferramenta de CI/dev para imposição da regra de Single Export Pattern.
//!
//! # Lacunas Conhecidas
//! - Itens gerados por macros procedurais (ex.: `tokio::task_local!` em `session.rs`)
//!   são invisíveis ao `syn` e não são contados.

#![allow(clippy::disallowed_macros, reason = "xtask é ferramenta de CLI")]

use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use syn::spanned::Spanned;

#[derive(Debug, Deserialize)]
struct AllowEntry {
    max: usize,
    reason: String,
    expires: String,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 || args[1] != "lint-exports" {
        eprintln!("Uso: cargo xtask lint-exports [--github]");
        return ExitCode::FAILURE;
    }

    let github = args.iter().any(|a| a == "--github");
    let allowlist_path = Path::new("xtask/exports-allow.toml");
    let allowlist: HashMap<String, AllowEntry> = if allowlist_path.exists() {
        let content = fs::read_to_string(allowlist_path).unwrap_or_default();
        toml::from_str(&content).unwrap_or_else(|e| {
            eprintln!("Erro ao ler xtask/exports-allow.toml: {e}");
            HashMap::new()
        })
    } else {
        HashMap::new()
    };

    let mut files = Vec::new();
    find_rs_files(Path::new("crates"), &mut files);
    files.sort();

    // A data real, e não uma constante: uma data fixa aqui faria toda entrada da
    // allowlist ser eterna — que é justamente o que o campo `expires` existe
    // para impedir. O formato ISO ordena lexicograficamente, então a comparação
    // de strings mais abaixo é a comparação de datas.
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let today = today.as_str();
    let mut failures = 0;

    for path in &files {
        let rel_path = path.to_string_lossy().to_string();
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Erro ao ler {rel_path}: {e}");
                failures += 1;
                continue;
            }
        };

        let syntax = match syn::parse_file(&content) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Erro ao analisar sintaxe de {rel_path}: {e}");
                failures += 1;
                continue;
            }
        };

        let exports = count_exports(&syntax);
        let export_count = exports.len();

        if let Some(entry) = allowlist.get(&rel_path) {
            if entry.expires.as_str() < today {
                let msg = format!(
                    "Allowlist expirada em {} (motivo: {})",
                    entry.expires, entry.reason
                );
                print_error(&rel_path, 1, &msg, github);
                failures += 1;
            }

            if export_count > entry.max {
                let msg = format!(
                    "Arquivo exporta {} itens, mas o teto na allowlist é {} (motivo: {})",
                    export_count, entry.max, entry.reason
                );
                print_error(&rel_path, 1, &msg, github);
                failures += 1;
            }

            if export_count <= 1 {
                let msg = format!(
                    "Entrada obsoleta na allowlist: arquivo agora exporta {} itens (<= 1). Remova a entrada em xtask/exports-allow.toml",
                    export_count
                );
                print_error(&rel_path, 1, &msg, github);
                failures += 1;
            }
        } else if export_count > 1 {
            let names: Vec<String> = exports.iter().map(|(name, _line)| name.clone()).collect();
            let line = exports.first().map(|(_n, line)| *line).unwrap_or(1);
            let msg = format!(
                "Single Export Pattern violado: {} itens exportados ({})",
                export_count,
                names.join(", ")
            );
            print_error(&rel_path, line, &msg, github);
            failures += 1;
        }
    }

    if failures > 0 {
        eprintln!("\nlint-exports falhou com {failures} erro(s).");
        ExitCode::FAILURE
    } else {
        println!(
            "lint-exports OK: todos os {} arquivos respeitam o limite.",
            files.len()
        );
        ExitCode::SUCCESS
    }
}

fn find_rs_files(dir: &Path, acc: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                find_rs_files(&path, acc);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                acc.push(path);
            }
        }
    }
}

fn print_error(file: &str, line: usize, msg: &str, github: bool) {
    if github {
        println!("::error file={file},line={line}::{msg}");
    } else {
        eprintln!("{file}:{line}: erro: {msg}");
    }
}

/// Os itens que o arquivo expõe, na ordem em que aparecem.
///
/// Itens com o **mesmo nome** contam uma vez só: é o caso de duas definições
/// mutuamente exclusivas por `#[cfg(feature = …)]`, que são um item para quem lê
/// e para quem compila.
fn count_exports(file: &syn::File) -> Vec<(String, usize)> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut result = Vec::new();

    for item in &file.items {
        if is_cfg_test(item) {
            continue;
        }

        if let Some((name, vis)) = get_item_name_and_vis(item) {
            if is_public_or_crate(&vis) && seen.insert(name.clone()) {
                result.push((name, get_item_line(item)));
            }
        }
    }

    result
}

fn is_cfg_test(item: &syn::Item) -> bool {
    let attrs = match item {
        syn::Item::Const(i) => &i.attrs,
        syn::Item::Enum(i) => &i.attrs,
        syn::Item::Fn(i) => &i.attrs,
        syn::Item::Macro(i) => &i.attrs,
        syn::Item::Mod(i) => &i.attrs,
        syn::Item::Static(i) => &i.attrs,
        syn::Item::Struct(i) => &i.attrs,
        syn::Item::Trait(i) => &i.attrs,
        syn::Item::Type(i) => &i.attrs,
        syn::Item::Union(i) => &i.attrs,
        _ => return false,
    };

    for attr in attrs {
        if attr.path().is_ident("cfg") {
            if let syn::Meta::List(list) = &attr.meta {
                if list.tokens.to_string().contains("test") {
                    return true;
                }
            }
        }
    }

    false
}

fn is_public_or_crate(vis: &syn::Visibility) -> bool {
    match vis {
        syn::Visibility::Public(_) => true,
        syn::Visibility::Restricted(r) => r.path.is_ident("crate"),
        syn::Visibility::Inherited => false,
    }
}

fn get_item_name_and_vis(item: &syn::Item) -> Option<(String, syn::Visibility)> {
    match item {
        syn::Item::Struct(i) => Some((i.ident.to_string(), i.vis.clone())),
        syn::Item::Enum(i) => Some((i.ident.to_string(), i.vis.clone())),
        syn::Item::Union(i) => Some((i.ident.to_string(), i.vis.clone())),
        syn::Item::Trait(i) => Some((i.ident.to_string(), i.vis.clone())),
        syn::Item::Fn(i) => Some((i.sig.ident.to_string(), i.vis.clone())),
        syn::Item::Const(i) => Some((i.ident.to_string(), i.vis.clone())),
        syn::Item::Static(i) => Some((i.ident.to_string(), i.vis.clone())),
        syn::Item::Type(i) => Some((i.ident.to_string(), i.vis.clone())),
        syn::Item::Mod(i) if i.content.is_some() => Some((i.ident.to_string(), i.vis.clone())),
        syn::Item::Macro(i) => {
            let has_macro_export = i.attrs.iter().any(|a| a.path().is_ident("macro_export"));
            if has_macro_export {
                let name = i
                    .ident
                    .as_ref()
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "macro".to_string());
                Some((name, syn::Visibility::Public(syn::token::Pub::default())))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn get_item_line(item: &syn::Item) -> usize {
    let span = match item {
        syn::Item::Struct(i) => i.span(),
        syn::Item::Enum(i) => i.span(),
        syn::Item::Union(i) => i.span(),
        syn::Item::Trait(i) => i.span(),
        syn::Item::Fn(i) => i.span(),
        syn::Item::Const(i) => i.span(),
        syn::Item::Static(i) => i.span(),
        syn::Item::Type(i) => i.span(),
        syn::Item::Mod(i) => i.span(),
        syn::Item::Macro(i) => i.span(),
        _ => return 1,
    };
    span.start().line
}
