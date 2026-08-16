//! A implementação das regras de autenticação.

use crate::domain::User;
use crate::error::AuthError;
use crate::security::PasswordHasher;
use crate::table_modules::AuthTM;

/// Monta as regras de autenticação com o seu hasher.
///
/// O hasher chega injetado e o que sai é o contrato: o tipo concreto não tem
/// nome fora deste arquivo.
pub(crate) fn auth_tm<H>(password_hasher: H) -> impl AuthTM + Send + Sync + Clone + use<H> + 'static
where
    H: PasswordHasher + Send + Sync + Clone + 'static,
{
    AuthTMImpl { password_hasher }
}

/// A implementação, genérica sobre o hasher.
#[derive(Clone)]
struct AuthTMImpl<H> {
    /// Quem confere a senha apresentada contra o hash gravado.
    password_hasher: H,
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
