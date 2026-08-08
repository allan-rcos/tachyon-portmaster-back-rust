//! As regras de autenticação.

use crate::error::AuthError;
use crate::models::User;

/// Confere credenciais.
pub trait AuthTM {
    /// Verifica a senha contra o hash guardado do usuário.
    fn login(&self, user: &dyn User, password: &str) -> Result<(), AuthError>;
}
