//! A entity de usuário.

use chrono::{DateTime, Utc};
use mysql_async::prelude::FromRow;
use mysql_async::{FromRowError, Row, Value};
use portmaster_domain::domain::Role;
use portmaster_domain::domain::User;

use crate::entity::decode::Decode;
use crate::entity::entity_id::EntityId;

/// A entity, que é também a linha de `users`.
///
/// Os papéis chegam separados: vêm de `user_roles` e são carregados pelo
/// repositório, não pela linha de `users`.
pub struct UserEntity {
    /// A identidade, nas duas formas.
    id: EntityId,
    /// Nome de exibição.
    name: String,
    /// E-mail, que também é a credencial de login.
    email: String,
    /// O hash Argon2, como está gravado.
    password_hash: String,
    /// Os papéis já hidratados.
    ///
    /// Não é coluna: os papéis vêm de `user_roles`, numa segunda consulta, e são
    /// anexados por [`with_roles`](Self::with_roles). Um usuário sem eles não
    /// está completo — é o repositório que fecha isso, porque é ele quem sabe
    /// consultar.
    ///
    /// É este campo que obriga o [`FromRow`] a ser escrito à mão: o derive lê a
    /// linha campo a campo e não tem como pular um.
    roles: Vec<Box<dyn Role>>,
    /// Quando a linha nasceu, em UTC.
    created_at: DateTime<Utc>,
    /// Quando a linha mudou pela última vez, em UTC.
    updated_at: DateTime<Utc>,
    /// Quando foi removida, ou `None` se ativa — o soft-delete.
    deleted_at: Option<DateTime<Utc>>,
}

impl FromRow for UserEntity {
    /// Lê a linha de `users`, com a lista de papéis ainda vazia.
    fn from_row_opt(row: Row) -> Result<Self, FromRowError> {
        Self::of(&row).ok_or(FromRowError(row))
    }
}

impl UserEntity {
    /// A entity, quando todas as colunas estão na linha e todas convertem.
    ///
    /// Devolve `None` na primeira que não converter, e é o `from_row_opt` que
    /// transforma isso no erro que carrega a linha inteira — que é onde quem lê
    /// enxerga qual valor recusou.
    fn of(row: &Row) -> Option<Self> {
        Some(Self {
            id: Decode::entity_id(Self::column(row, "id")?).ok()?,
            name: row.get_opt("name")?.ok()?,
            email: row.get_opt("email")?.ok()?,
            password_hash: row.get_opt("password_hash")?.ok()?,
            roles: Vec::new(),
            created_at: Decode::utc(Self::column(row, "created_at")?).ok()?,
            updated_at: Decode::utc(Self::column(row, "updated_at")?).ok()?,
            deleted_at: Decode::utc_opt(Self::column(row, "deleted_at")?).ok()?,
        })
    }

    /// A coluna crua, para as conversões que o [`Decode`] faz.
    fn column(row: &Row, name: &str) -> Option<Value> {
        row.get_opt::<Value, _>(name)?.ok()
    }

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
