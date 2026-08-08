//! A entity de papel.

use crate::entity::codec::Codec;
use crate::entity::role_row::RoleRow;
use chrono::{DateTime, Utc};
use portmaster_domain::models::Role;

/// A entity, com o id já traduzido para base62.
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
    ///
    /// Uma coluna JSON ilegível é linha corrompida. Assumir lista vazia
    /// silenciosamente transformaria isso numa revogação de todas as
    /// permissões do papel, que é o pior desfecho possível.
    /// Reconstrói a entity a partir de uma linha lida.
    pub(crate) fn from_row(row: RoleRow) -> anyhow::Result<Self> {
        let permissions: Vec<String> = serde_json::from_str(&row.permissions)
            .map_err(|e| anyhow::anyhow!("permissões do papel {} ilegíveis: {e}", row.id))?;

        Ok(Self {
            id: Codec::encode_id(row.id),
            raw_id: row.id,
            name: row.name,
            permissions,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        })
    }

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
