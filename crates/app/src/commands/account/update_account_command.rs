//! Alterar o próprio perfil.

use crate::context::UserContext;

/// Alterar o próprio nome e e-mail.
#[derive(Debug, Clone)]
pub struct UpdateAccountCommand {
    /// Quem está agindo — e sobre quem se age.
    pub context: UserContext,
    /// Nome do usuário.
    pub name: String,
    /// E-mail do usuário.
    pub email: String,
}
