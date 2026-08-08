//! Os parâmetros da listagem de resumos de contêiner.

/// Os filtros da listagem de resumos de contêiner.
#[derive(Debug, Clone, Default)]
pub struct SummaryListParams {
    /// Restringe a um contêiner, em base62.
    pub id: Option<String>,
    /// Token da página anterior; ausente pede a primeira.
    pub cursor: Option<String>,
    /// Tamanho da página; ausente usa [`DEFAULT_LIMIT`](crate::query::default_limit::DEFAULT_LIMIT).
    pub limit: Option<u32>,
}
