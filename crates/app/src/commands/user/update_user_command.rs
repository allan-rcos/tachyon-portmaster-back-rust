//! Alterar um usuário.

use crate::context::UserContext;

/// Alterar nome e e-mail de um usuário.
#[derive(Debug, Clone)]
pub struct UpdateUserCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Id do usuário, em base62.
    pub id: String,
    /// Nome do usuário.
    pub name: String,
    /// E-mail do usuário.
    pub email: String,
}
