//! Os filtros da listagem de resumos.

use serde::Deserialize;

/// Os filtros da listagem de resumos.
#[derive(Debug, Default, Deserialize)]
pub struct SummaryPageParams {
    /// Restringe a um contêiner.
    pub(crate) id: Option<String>,
    /// Token da página anterior.
    pub(crate) cursor: Option<String>,
    /// Tamanho da página.
    pub(crate) limit: Option<u32>,
}
