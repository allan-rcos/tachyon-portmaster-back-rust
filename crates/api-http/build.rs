//! Geração dos tipos de wire FlatBuffers.
//!
//! Os `.fbs` de `swagger/flatbuffers/schemas/` são compilados **em Rust**, pelo
//! planus, para `OUT_DIR`. Não há shell-out para o binário `flatc`: ele não
//! existe em produção, e depender dele tornaria o build da API refém de uma
//! ferramenta externa instalada à parte.
//!
//! Os schemas são a fonte compartilhada com o cliente e não são tocados por este
//! passo — o que sai daqui é só a tradução deles para Rust.

use std::path::{Path, PathBuf};

/// Onde vivem os schemas, relativo à raiz do workspace.
const SCHEMA_DIR: &str = "../../swagger/flatbuffers/schemas";

fn main() -> anyhow::Result<()> {
    let schema_dir = Path::new(SCHEMA_DIR);
    println!("cargo:rerun-if-changed={SCHEMA_DIR}");

    let mut schemas: Vec<PathBuf> = std::fs::read_dir(schema_dir)
        .map_err(|e| anyhow::anyhow!("não foi possível ler {SCHEMA_DIR}: {e}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "fbs"))
        .collect();

    // Ordem estável: o planus resolve os `include` sozinho, mas gerar na ordem do
    // sistema de arquivos faria a saída variar entre máquinas e sujar os diffs.
    schemas.sort();

    if schemas.is_empty() {
        anyhow::bail!("nenhum .fbs encontrado em {SCHEMA_DIR}");
    }

    for schema in &schemas {
        println!("cargo:rerun-if-changed={}", schema.display());
    }

    let declarations = planus_translation::translate_files(&schemas).ok_or_else(|| {
        // O planus já escreveu o diagnóstico detalhado no stderr do build.
        anyhow::anyhow!("os schemas FlatBuffers não passaram na validação do planus")
    })?;

    let generated = planus_codegen::generate_rust(&declarations, true)
        .map_err(|e| anyhow::anyhow!("falha ao gerar o Rust a partir dos schemas: {e}"))?;

    let out = PathBuf::from(std::env::var("OUT_DIR")?).join("wire_generated.rs");
    std::fs::write(&out, generated)
        .map_err(|e| anyhow::anyhow!("falha ao escrever {}: {e}", out.display()))?;

    publish_rustc_version();

    Ok(())
}

/// Grava a versão do compilador numa variável, para o `GET /info`.
///
/// O binário não tem como se perguntar em runtime com que rustc foi compilado, e
/// `CARGO_PKG_RUST_VERSION` responderia a **outra** pergunta: ela é o mínimo que
/// o crate aceita, não o que de fato construiu este executável. Quem lê `/info`
/// quer saber o que está rodando.
///
/// Se o `rustc` não responder, a variável fica vazia e o handler diz apenas
/// "Rust" — um `/info` sem o número ainda serve para o que a rota existe.
fn publish_rustc_version() {
    let version = std::env::var("RUSTC")
        .ok()
        .and_then(|rustc| {
            std::process::Command::new(rustc)
                .arg("--version")
                .output()
                .ok()
        })
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|line| line.trim().to_owned())
        .unwrap_or_default();

    println!("cargo:rustc-env=PORTMASTER_RUSTC_VERSION={version}");
}
