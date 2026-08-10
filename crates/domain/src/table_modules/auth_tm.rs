//! As regras de autenticação.

use crate::domain::User;
use crate::error::AuthError;

/// Confere credenciais.
pub trait AuthTM {
    /// Verifica a senha contra o hash guardado do usuário.
    fn login(&self, user: &dyn User, password: &str) -> Result<(), AuthError>;
}
