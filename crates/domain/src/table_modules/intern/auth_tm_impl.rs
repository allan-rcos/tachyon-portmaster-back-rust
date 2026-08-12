//! A implementação das regras de autenticação.

use crate::domain::User;
use crate::error::AuthError;
use crate::security::PasswordHasher;
use crate::table_modules::AuthTM;

/// A implementação, genérica sobre o hasher.
#[derive(Clone)]
pub(crate) struct AuthTMImpl<H> {
    /// Quem confere a senha apresentada contra o hash gravado.
    password_hasher: H,
}

impl<H: PasswordHasher> AuthTMImpl<H> {
    /// Monta o `TableModule` com o seu hasher.
    pub(crate) const fn new(password_hasher: H) -> Self {
        Self { password_hasher }
    }
}

impl<H: PasswordHasher> AuthTM for AuthTMImpl<H> {
    /// Confere a credencial apresentada.
    ///
    /// Há um erro só, e ele não diz se o problema foi o e-mail ou a senha:
    /// distinguir os dois confirmaria a existência da conta para quem está
    /// sondando.
    fn login(&self, user: &dyn User, password: &str) -> Result<(), AuthError> {
        if self.password_hasher.verify(password, user.password_hash()) {
            return Ok(());
        }

        Err(AuthError::InvalidCredentials)
    }
}

#[cfg(test)]
#[path = "tests/auth_tm_impl_test.rs"]
mod tests;
