//! Listar papéis.

use crate::context::UserContext;

/// Listar papéis.
#[derive(Debug, Clone)]
pub struct ListRolesQuery {
    /// Quem está consultando.
    pub context: UserContext,
    /// Token da página anterior.
    pub cursor: Option<String>,
    /// Tamanho da página.
    pub limit: Option<u32>,
    /// Termo de busca.
    pub search: Option<String>,
}
