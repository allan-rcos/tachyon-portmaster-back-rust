//! Remover um usuário.

use crate::context::UserContext;

/// Remover um usuário.
#[derive(Debug, Clone)]
pub struct DeleteUserCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Id do usuário, em base62.
    pub id: String,
}
