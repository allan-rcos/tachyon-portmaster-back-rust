//! Um usuário de domínio, montado para o teste.

use chrono::{DateTime, TimeZone as _, Utc};
use portmaster_domain::domain::{Role, User};

/// Um usuário que o teste controla.
pub(crate) struct StubUser {
    /// Identidade em base62.
    id: String,
    /// E-mail, que também é a credencial.
    email: String,
    /// Os papéis que ele carrega.
    roles: Vec<Box<dyn Role>>,
}

impl StubUser {
    /// O usuário deste id, dentro do `Box` que o table module devolveria.
    pub(crate) fn boxed(id: &str, email: &str) -> Box<dyn User> {
        Box::new(Self {
            id: id.to_owned(),
            email: email.to_owned(),
            roles: Vec::new(),
        })
    }
}

/// O instante fixo das datas.
///
/// Fixo porque nenhuma asserção depende delas: o que se testa é orquestração, e
/// uma data variável só faria o teste parecer sensível ao relógio.
fn epoch() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
        .single()
        .expect("a data é válida e não é ambígua")
}

impl User for StubUser {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &'static str {
        "Usuário do teste"
    }

    fn email(&self) -> &str {
        &self.email
    }

    fn password_hash(&self) -> &'static str {
        "$argon2id$v=19$m=1,t=1,p=1$c2FsdA$hash"
    }

    fn roles(&self) -> &[Box<dyn Role>] {
        &self.roles
    }

    fn created_at(&self) -> DateTime<Utc> {
        epoch()
    }

    fn updated_at(&self) -> DateTime<Utc> {
        epoch()
    }

    fn deleted_at(&self) -> Option<DateTime<Utc>> {
        None
    }
}
