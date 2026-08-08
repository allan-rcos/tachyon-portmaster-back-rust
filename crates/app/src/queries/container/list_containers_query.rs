//! Listar contêineres.

use crate::context::UserContext;
use portmaster_domain::enums::ContainerStatus;

/// Listar contêineres.
#[derive(Debug, Clone)]
pub struct ListContainersQuery {
    /// Quem está consultando.
    pub context: UserContext,
    /// Token da página anterior.
    pub cursor: Option<String>,
    /// Tamanho da página.
    pub limit: Option<u32>,
    /// Termo de busca sobre o código.
    pub search: Option<String>,
    /// Restringe a um status.
    pub status: Option<ContainerStatus>,
    /// Restringe a um conjunto de status.
    pub status_in: Vec<ContainerStatus>,
}
