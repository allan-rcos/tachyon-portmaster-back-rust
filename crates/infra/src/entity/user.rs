//! A entity de usuário.

use chrono::{DateTime, Utc};
use portmaster_domain::role::Role;
use portmaster_domain::user::User;

use super::{decode_id, encode_id};

/// Uma linha de `users`.
#[derive(sqlx::FromRow)]
pub(crate) struct UserRow {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) password_hash: String,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) deleted_at: Option<DateTime<Utc>>,
}

/// A entity, com o id já traduzido para base62.
///
/// Os papéis chegam separados: vêm de `user_roles` e são carregados pelo
/// repositório, não pela linha de `users`.
pub(crate) struct UserEntity {
    id: String,
    raw_id: i64,
    name: String,
    email: String,
    password_hash: String,
    roles: Vec<Box<dyn Role>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl UserEntity {
    /// Reconstrói a entity a partir de uma linha lida e dos papéis já buscados.
    pub(crate) fn from_row(row: UserRow, roles: Vec<Box<dyn Role>>) -> Self {
        Self {
            id: encode_id(row.id),
            raw_id: row.id,
            name: row.name,
            email: row.email,
            password_hash: row.password_hash,
            roles,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        }
    }

    /// Recria a entity a partir de qualquer [`User`], para gravá-la.
    pub(crate) fn from_domain(source: &dyn User) -> anyhow::Result<Self> {
        Ok(Self {
            id: source.id().to_owned(),
            raw_id: decode_id(source.id())?,
            name: source.name().to_owned(),
            email: source.email().to_owned(),
            password_hash: source.password_hash().to_owned(),
            roles: source.roles().iter().map(|r| r.clone_role()).collect(),
            created_at: source.created_at(),
            updated_at: source.updated_at(),
            deleted_at: source.deleted_at(),
        })
    }

    /// O id como o banco o guarda.
    pub(crate) fn raw_id(&self) -> i64 {
        self.raw_id
    }
}

impl User for UserEntity {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn email(&self) -> &str {
        &self.email
    }

    fn password_hash(&self) -> &str {
        &self.password_hash
    }

    fn roles(&self) -> &[Box<dyn Role>] {
        &self.roles
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }
}
