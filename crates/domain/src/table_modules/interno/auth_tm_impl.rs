//! A implementação das regras de autenticação.

use crate::error::AuthError;
use crate::models::User;
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
mod tests {
    use super::*;
    use crate::models::Role;
    use crate::security::interno::argon2_hasher::Argon2Hasher;
    use chrono::{DateTime, Utc};

    /// Usuário mínimo, com o hash que o teste quiser.
    struct StubUser {
        password_hash: String,
    }

    impl User for StubUser {
        fn id(&self) -> &'static str {
            "U1"
        }
        fn name(&self) -> &'static str {
            "Ana"
        }
        fn email(&self) -> &'static str {
            "ana@portmaster.local"
        }
        fn password_hash(&self) -> &str {
            &self.password_hash
        }
        fn roles(&self) -> &[Box<dyn Role>] {
            &[]
        }
        fn created_at(&self) -> DateTime<Utc> {
            Utc::now()
        }
        fn updated_at(&self) -> DateTime<Utc> {
            Utc::now()
        }
        fn deleted_at(&self) -> Option<DateTime<Utc>> {
            None
        }
    }

    fn user_with_password(password: &str) -> StubUser {
        StubUser {
            password_hash: Argon2Hasher::new().hash(password),
        }
    }

    #[test]
    fn aceita_a_senha_correta() {
        let table_module = AuthTMImpl::new(Argon2Hasher::new());
        let user = user_with_password("Portmaster1");

        assert!(table_module.login(&user, "Portmaster1").is_ok());
    }

    #[test]
    fn recusa_a_senha_errada() {
        let table_module = AuthTMImpl::new(Argon2Hasher::new());
        let user = user_with_password("Portmaster1");

        assert!(matches!(
            table_module.login(&user, "Portmaster2"),
            Err(AuthError::InvalidCredentials)
        ));
    }

    #[test]
    fn hash_corrompido_nao_autentica() {
        // Uma linha danificada no banco não pode virar uma porta aberta.
        let table_module = AuthTMImpl::new(Argon2Hasher::new());
        let user = StubUser {
            password_hash: "isto não é um hash".into(),
        };

        assert!(table_module.login(&user, "qualquer").is_err());
    }
}
