//! Os testes de `auth_tm_impl`.

use super::*;
use crate::domain::Role;
use crate::security::intern::argon2_hasher::Argon2Hasher;
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
