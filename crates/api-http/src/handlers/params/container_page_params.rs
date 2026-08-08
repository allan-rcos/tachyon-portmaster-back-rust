//! Os filtros da listagem de contêineres.

use serde::Deserialize;

/// Os filtros da listagem de contêineres.
#[derive(Debug, Default, Deserialize)]
pub struct ContainerPageParams {
    /// Token da página anterior.
    pub(crate) cursor: Option<String>,
    /// Tamanho da página.
    pub(crate) limit: Option<u32>,
    /// Termo de busca sobre o código.
    pub(crate) search: Option<String>,
    /// Restringe a um status, pelo nome do enum.
    pub(crate) status: Option<String>,
    /// Restringe a um conjunto de status, separados por vírgula.
    pub(crate) status_in: Option<String>,
}
