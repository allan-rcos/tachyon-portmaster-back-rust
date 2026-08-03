//! A regra de autenticação: a senha confere?
//!
//! Só isso. Quem é o usuário, se a sessão existe, o que vira token — nada disso
//! é do domínio. Ele responde a uma pergunta de negócio e devolve.

use crate::error::AuthError;
use crate::security::PasswordHasher;
use crate::user::User;

/// Confere credenciais.
pub trait AuthTM {
    /// Verifica a senha contra o hash guardado do usuário.
    fn login(&self, user: &dyn User, password: &str) -> Result<(), AuthError>;
}

/// A implementação, genérica sobre o hasher.
pub(crate) struct AuthTMImpl<H> {
    password_hasher: H,
}

impl<H: PasswordHasher> AuthTMImpl<H> {
    /// Monta o TableModule com o seu hasher.
    pub(crate) fn new(password_hasher: H) -> Self {
        Self { password_hasher }
    }
}

impl<H: PasswordHasher> AuthTM for AuthTMImpl<H> {
    fn login(&self, user: &dyn User, password: &str) -> Result<(), AuthError> {
        if self.password_hasher.verify(password, user.password_hash()) {
            return Ok(());
        }

        // Um erro só, sem dizer se o problema foi o e-mail ou a senha:
        // distinguir os dois confirmaria a existência da conta para quem está
        // sondando.
        Err(AuthError::InvalidCredentials)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::role::Role;
    use crate::security::argon2::Argon2Hasher;
    use chrono::{DateTime, Utc};

    /// Usuário mínimo, com o hash que o teste quiser.
    struct StubUser {
        password_hash: String,
    }

    impl User for StubUser {
        fn id(&self) -> &str {
            "U1"
        }
        fn name(&self) -> &str {
            "Ana"
        }
        fn email(&self) -> &str {
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
