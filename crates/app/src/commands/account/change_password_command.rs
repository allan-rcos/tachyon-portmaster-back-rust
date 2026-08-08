//! Trocar a própria senha.

use crate::context::UserContext;

/// Trocar a própria senha.
#[derive(Debug, Clone)]
pub struct ChangePasswordCommand {
    /// Quem está agindo — e sobre quem se age.
    pub context: UserContext,
    /// A senha atual, para provar posse da conta.
    pub current_password: String,
    /// A senha nova.
    pub new_password: String,
}
