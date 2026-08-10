//! As decisões que toda listagem repete.

use crate::query::DEFAULT_LIMIT;
use crate::search_key::SearchKey;

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

    /// Quantas linhas pular. Página ausente ou zero é a primeira.
    ///
    /// O `page` vem da query string, então é um `u32` arbitrário que o cliente
    /// escolhe. Multiplicá-lo pelo limite estoura o `u32` a partir de ~86
    /// milhões de páginas — em release isso daria a volta e devolveria a página
    /// errada em silêncio. Saturar é o comportamento certo: pedir uma página
    /// além do fim é uma lista vazia, não um resultado sorteado.
    pub fn offset(page: Option<u32>, limit: u32) -> u32 {
        let page = page.filter(|value| *value > 0).unwrap_or(1);

        page.saturating_sub(1).saturating_mul(limit)
    }
}
