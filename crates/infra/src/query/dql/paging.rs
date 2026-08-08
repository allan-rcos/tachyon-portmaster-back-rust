//! Os descritores de consulta.
//!
//! Um por leitura que a API oferece. Cada um carrega os próprios parâmetros,
//! monta o próprio SQL e hidrata a própria View — o que deixa o
//! [`QueryRepository`](crate::query::query_repository::QueryRepository) sem nada específico a saber.
//!
//! Todos são `pub(crate)`. O `app` os alcança pela
//! [`QueryFactory`](crate::query::query_factory::QueryFactory), que devolve `impl SqlDql<View = …>`:
//! executável, mas opaco. É o que mantém a montagem de consulta desta camada
//! para dentro.

use crate::query::DEFAULT_LIMIT;
use crate::text::search_key::SearchKey;

/// As decisões de paginação e busca que todo DQL de listagem repete.
///
/// Namespace: limite efetivo, termo normalizado e o `LIKE` escapado são a mesma
/// regra em dez consultas, e mudá-la num lugar só é o ponto.
pub(crate) struct Paging;

impl Paging {
    /// O limite que vale de fato.
    ///
    /// Zero cai no padrão junto com o ausente: um `LIMIT 0` devolveria página vazia
    /// para sempre, e ninguém pede isso de propósito.
    pub fn effective_limit(limit: Option<u32>) -> u32 {
        limit.filter(|value| *value > 0).unwrap_or(DEFAULT_LIMIT)
    }

    /// O termo de busca reduzido à chave que as colunas `search_*` guardam.
    ///
    /// Busca em branco é o mesmo que busca ausente — filtrar por string vazia
    /// casaria com tudo e ainda custaria a varredura.
    pub fn normalized_search(search: Option<&str>) -> Option<String> {
        let term = search?.trim();

        (!term.is_empty()).then(|| SearchKey::of(term))
    }

    /// O termo como o `LIKE` o quer.
    pub fn like(term: &str) -> String {
        format!("%{term}%")
    }
}
