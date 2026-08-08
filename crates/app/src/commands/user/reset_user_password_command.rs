//! Redefinir a senha de um usuário.

use crate::context::UserContext;

/// Redefinir a senha de um usuário.
///
/// É o caminho administrativo: não pede a senha atual, porque quem a executa não
/// a conhece. Trocar a própria senha é [`ChangePasswordCommand`](crate::commands::account::ChangePasswordCommand),
/// que exige a atual.
#[derive(Debug, Clone)]
pub struct ResetUserPasswordCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Id do usuário, em base62.
    pub id: String,
    /// A senha nova.
    pub new_password: String,
}
