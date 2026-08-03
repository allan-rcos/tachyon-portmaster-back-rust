//! Os descritores de consulta.
//!
//! Um por leitura que a API oferece. Cada um carrega os próprios parâmetros,
//! monta o próprio SQL e hidrata a própria View — o que deixa o
//! [`QueryRepository`](super::QueryRepository) sem nada específico a saber.
//!
//! Todos são `pub(crate)`. O `app` os alcança pela
//! [`QueryFactory`](super::QueryFactory), que devolve `impl SqlDql<View = …>`:
//! executável, mas opaco. É o que mantém a montagem de consulta desta camada
//! para dentro.

pub(crate) mod account;
pub(crate) mod container;
pub(crate) mod metrics;
pub(crate) mod product;
pub(crate) mod role;

use crate::query::DEFAULT_LIMIT;
use crate::text::search_key;

/// O limite que vale de fato.
///
/// Zero cai no padrão junto com o ausente: um `LIMIT 0` devolveria página vazia
/// para sempre, e ninguém pede isso de propósito.
pub(crate) fn effective_limit(limit: Option<u32>) -> u32 {
    limit.filter(|value| *value > 0).unwrap_or(DEFAULT_LIMIT)
}

/// O termo de busca reduzido à chave que as colunas `search_*` guardam.
///
/// Busca em branco é o mesmo que busca ausente — filtrar por string vazia
/// casaria com tudo e ainda custaria a varredura.
pub(crate) fn normalized_search(search: Option<&str>) -> Option<String> {
    let term = search?.trim();

    (!term.is_empty()).then(|| search_key(term))
}

/// O termo como o `LIKE` o quer.
pub(crate) fn like(term: &str) -> String {
    format!("%{term}%")
}
