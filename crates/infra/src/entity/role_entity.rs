//! A entity de papel.

use chrono::{DateTime, Utc};
use portmaster_domain::domain::Role;
use sqlx::mysql::MySqlRow;
use sqlx::{FromRow, Row as _};

use crate::entity::codec::Codec;

/// A entity, com o id já traduzido para base62.
///
/// É ela quem implementa [`FromRow`]: a linha crua não vira uma struct própria
/// antes de virar entity, porque essa struct não tinha comportamento nenhum.
pub struct RoleEntity {
    /// Identidade em base62.
    id: String,
    /// O mesmo id como `BIGINT`, para os `WHERE` e as FKs.
    ///
    /// Guardado junto do base62 para que a escrita não precise decodificar de
    /// volta a cada consulta.
    raw_id: i64,
    /// Nome do papel.
    name: String,
    /// Os slugs concedidos, já decodificados da coluna `JSON`.
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
            id: source.id().to_owned(),
            raw_id: Codec::decode_id(source.id())?,
            name: source.name().to_owned(),
            permissions: source.permissions().to_vec(),
            created_at: source.created_at(),
            updated_at: source.updated_at(),
            deleted_at: source.deleted_at(),
        })
    }

    /// O id como o banco o guarda.
    pub(crate) const fn raw_id(&self) -> i64 {
        self.raw_id
    }

    /// As permissões como a coluna `JSON` as guarda.
    pub(crate) fn permissions_json(&self) -> anyhow::Result<String> {
        serde_json::to_string(&self.permissions)
            .map_err(|e| anyhow::anyhow!("falha ao serializar as permissões do papel: {e}"))
    }
}

impl FromRow<'_, MySqlRow> for RoleEntity {
    /// Uma linha de `roles` como a entity a quer.
    ///
    /// Uma coluna JSON ilegível é linha corrompida, e falha. Assumir lista vazia
    /// silenciosamente transformaria isso numa revogação de todas as permissões
    /// do papel, que é o pior desfecho possível.
    fn from_row(row: &MySqlRow) -> sqlx::Result<Self> {
        let raw_id: i64 = row.try_get("id")?;
        let raw_permissions: String = row.try_get("permissions")?;

        Ok(Self {
            id: Codec::encode_id(raw_id),
            raw_id,
            name: row.try_get("name")?,
            permissions: serde_json::from_str(&raw_permissions).map_err(|error| {
                sqlx::Error::Decode(
                    format!("permissões do papel {raw_id} ilegíveis: {error}").into(),
                )
            })?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            deleted_at: row.try_get("deleted_at")?,
        })
    }
}

impl Role for RoleEntity {
    fn id(&self) -> &str {
        &self.id
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
        Box::new(Self {
            id: self.id.clone(),
            raw_id: self.raw_id,
            name: self.name.clone(),
            permissions: self.permissions.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
        })
    }
}
