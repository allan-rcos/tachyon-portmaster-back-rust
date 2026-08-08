//! Os parâmetros da listagem de contêineres.

use portmaster_domain::enums::ContainerStatus;

/// Os filtros da listagem de contêineres.
#[derive(Debug, Clone, Default)]
pub struct ContainerListParams {
    /// Token da página anterior; ausente pede a primeira.
    pub cursor: Option<String>,
    /// Tamanho da página; ausente usa [`DEFAULT_LIMIT`](crate::query::default_limit::DEFAULT_LIMIT).
    pub limit: Option<u32>,
    /// Termo de busca sobre o código, ainda como o cliente digitou.
    pub search: Option<String>,
    /// Restringe a um status.
    pub status: Option<ContainerStatus>,
    /// Restringe a um conjunto de status; vazio não filtra.
    pub status_in: Vec<ContainerStatus>,
}
