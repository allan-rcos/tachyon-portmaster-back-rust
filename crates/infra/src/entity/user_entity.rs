//! A entity de usuário.

use chrono::{DateTime, Utc};
use portmaster_domain::domain::Role;
use portmaster_domain::domain::User;
use sqlx::FromRow;

use crate::entity::entity_id::EntityId;

/// A entity, que é também a linha de `users`.
///
/// Os papéis chegam separados: vêm de `user_roles` e são carregados pelo
/// repositório, não pela linha de `users`.
#[derive(FromRow)]
pub struct UserEntity {
    /// A identidade, nas duas formas.
    #[sqlx(try_from = "i64")]
    id: EntityId,
    /// Nome de exibição.
    name: String,
    /// E-mail, que também é a credencial de login.
    email: String,
    /// O hash Argon2, como está gravado.
    password_hash: String,
    /// Os papéis já hidratados.
    ///
    /// `skip` e não `default`: os dois caem no `Default`, mas o `default` ainda
    /// tenta ler a coluna antes, e exigiria um `Decode` que `Box<dyn Role>` não
    /// tem. `skip` não toca na linha.
    ///
    /// Não é coluna porque os papéis vêm de `user_roles`, numa segunda consulta,
    /// e são anexados por [`with_roles`](Self::with_roles). Um usuário sem eles
    /// não está completo — é o repositório que fecha isso, porque é ele quem
    /// sabe consultar.
    #[sqlx(skip)]
    roles: Vec<Box<dyn Role>>,
    /// Quando a linha nasceu, em UTC.
    created_at: DateTime<Utc>,
    /// Quando a linha mudou pela última vez, em UTC.
    updated_at: DateTime<Utc>,
    /// Quando foi removida, ou `None` se ativa — o soft-delete.
    deleted_at: Option<DateTime<Utc>>,
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
            id: EntityId::try_from(source.id())?,
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
        self.id.raw()
    }
}

impl User for UserEntity {
    fn id(&self) -> &str {
        self.id.as_str()
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
