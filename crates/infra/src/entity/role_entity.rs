//! A entity de papel.

use chrono::{DateTime, Utc};
use portmaster_domain::domain::Role;
use sqlx::FromRow;

use crate::entity::entity_id::EntityId;

/// A entity, que é também a linha de `roles`.
///
/// O `FromRow` é derivado: cada conversão que antes se escrevia à mão no corpo
/// de um `from_row` virou atributo do campo que a exige — o id pelo
/// `try_from`, as permissões pelo `json`.
#[derive(Clone, FromRow)]
pub struct RoleEntity {
    /// A identidade, nas duas formas.
    #[sqlx(try_from = "i64")]
    id: EntityId,
    /// Nome do papel.
    name: String,
    /// Os slugs concedidos, na coluna `JSON`.
    ///
    /// Uma coluna ilegível é linha corrompida, e o `json` a faz falhar. Assumir
    /// lista vazia silenciosamente seria revogar todas as permissões do papel,
    /// que é o pior desfecho possível.
    #[sqlx(json)]
    permissions: Vec<String>,
    /// Quando a linha nasceu, em UTC.
    created_at: DateTime<Utc>,
    /// Quando a linha mudou pela última vez, em UTC.
    updated_at: DateTime<Utc>,
    /// Quando foi removida, ou `None` se ativa — o soft-delete.
    deleted_at: Option<DateTime<Utc>>,
}

impl RoleEntity {
    /// Recria a entity a partir de qualquer [`Role`], para gravá-la.
    pub(crate) fn from_domain(source: &dyn Role) -> anyhow::Result<Self> {
        Ok(Self {
            id: EntityId::try_from(source.id())?,
            name: source.name().to_owned(),
            permissions: source.permissions().to_vec(),
            created_at: source.created_at(),
            updated_at: source.updated_at(),
            deleted_at: source.deleted_at(),
        })
    }

    /// O id como o banco o guarda.
    pub(crate) const fn raw_id(&self) -> i64 {
        self.id.raw()
    }

    /// As permissões como a coluna `JSON` as guarda.
    pub(crate) fn permissions_json(&self) -> anyhow::Result<String> {
        serde_json::to_string(&self.permissions)
            .map_err(|e| anyhow::anyhow!("falha ao serializar as permissões do papel: {e}"))
    }
}

impl Role for RoleEntity {
    fn id(&self) -> &str {
        self.id.as_str()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn permissions(&self) -> &[String] {
        &self.permissions
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

    fn clone_role(&self) -> Box<dyn Role> {
        Box::new(self.clone())
    }
}
