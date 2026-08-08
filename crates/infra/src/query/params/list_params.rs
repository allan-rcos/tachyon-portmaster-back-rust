//! Os parâmetros de uma listagem paginada por cursor.

/// Os filtros de uma listagem paginada por cursor.
#[derive(Debug, Clone, Default)]
pub struct ListParams {
    /// Token da página anterior; ausente pede a primeira.
    pub cursor: Option<String>,
    /// Tamanho da página; ausente usa [`DEFAULT_LIMIT`](crate::query::default_limit::DEFAULT_LIMIT).
    pub limit: Option<u32>,
    /// Termo de busca, ainda como o cliente digitou.
    pub search: Option<String>,
}
