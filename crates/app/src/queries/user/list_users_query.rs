//! Listar usuários.

use crate::context::UserContext;

/// Listar usuários.
#[derive(Debug, Clone)]
pub struct ListUsersQuery {
    /// Quem está consultando.
    pub context: UserContext,
    /// Página, começando em 1.
    pub page: Option<u32>,
    /// Tamanho da página.
    pub limit: Option<u32>,
}
