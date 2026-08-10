//! A entity de usuário.

use chrono::{DateTime, Utc};
use portmaster_domain::domain::Role;
use portmaster_domain::domain::User;
use sqlx::mysql::MySqlRow;
use sqlx::{FromRow, Row as _};

use crate::entity::codec::Codec;

/// A entity, com o id já traduzido para base62.
///
/// Os papéis chegam separados: vêm de `user_roles` e são carregados pelo
/// repositório, não pela linha de `users`.
pub struct UserEntity {
    /// Identidade em base62, que é a forma que sai desta camada.
    id: String,
    /// O mesmo id como `BIGINT`, para os `WHERE` e as FKs.
    ///
    /// Guardado junto do base62 para que a escrita não precise decodificar de
    /// volta a cada consulta.
    raw_id: i64,
    /// Nome de exibição.
    name: String,
    /// E-mail, que também é a credencial de login.
    email: String,
    /// O hash Argon2, como está gravado.
    password_hash: String,
    /// Os papéis já hidratados, vindos da mesma consulta.
    roles: Vec<Box<dyn Role>>,
    /// Quando a linha nasceu, em UTC.
    created_at: DateTime<Utc>,
    /// Quando a linha mudou pela última vez, em UTC.
    updated_at: DateTime<Utc>,
    /// Quando foi removida, ou `None` se ativa — o soft-delete.
    deleted_at: Option<DateTime<Utc>>,
}

impl FromRow<'_, MySqlRow> for UserEntity {
    /// Uma linha de `users` como a entity a quer, **sem papéis**.
    ///
    /// Os papéis vêm de `user_roles`, numa segunda consulta, e são anexados por
    /// [`with_roles`](Self::with_roles). Um usuário sem eles não está completo —
    /// é o repositório que fecha isso, porque é ele quem sabe consultar.
    fn from_row(row: &MySqlRow) -> sqlx::Result<Self> {
        let raw_id: i64 = row.try_get("id")?;

        Ok(Self {
            id: Codec::encode_id(raw_id),
            raw_id,
            name: row.try_get("name")?,
            email: row.try_get("email")?,
            password_hash: row.try_get("password_hash")?,
            roles: Vec::new(),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            deleted_at: row.try_get("deleted_at")?,
        })
    }
}

impl UserEntity {
    /// Anexa os papéis que o repositório buscou.
    #[must_use]
    pub(crate) fn with_roles(mut self, roles: Vec<Box<dyn Role>>) -> Self {
        self.roles = roles;
        self
    }

    /// Recria a entity a partir de qualquer [`User`], para gravá-la.
    pub(crate) fn from_domain(source: &dyn User) -> anyhow::Result<Self> {
        Ok(Self {
            id: source.id().to_owned(),
            raw_id: Codec::decode_id(source.id())?,
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
    pub(crate) const fn raw_id(&self) -> i64 {
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
