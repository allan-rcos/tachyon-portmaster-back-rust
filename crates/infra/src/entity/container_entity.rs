//! A entity de contêiner.

use crate::entity::codec::Codec;
use crate::entity::container_row::ContainerRow;
use chrono::{DateTime, Utc};
use portmaster_domain::enums::ContainerStatus;
use portmaster_domain::models::Container;

/// A entity, com o id já traduzido para base62.
pub struct ContainerEntity {
    id: String,
    raw_id: i64,
    code: String,
    current_weight: f64,
    max_capacity: f64,
    status: ContainerStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl ContainerEntity {
    /// Reconstrói a entity a partir de uma linha lida.
    pub(crate) fn from_row(row: ContainerRow) -> anyhow::Result<Self> {
        Ok(Self {
            id: Codec::encode_id(row.id),
            raw_id: row.id,
            code: row.code,
            current_weight: row.current_weight,
            max_capacity: row.max_capacity,
            status: Codec::decode_enum(row.status, ContainerStatus::from_i32, "ContainerStatus")?,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        })
    }

    /// Recria a entity a partir de qualquer [`Container`], para gravá-la.
    pub(crate) fn from_domain(source: &dyn Container) -> anyhow::Result<Self> {
        Ok(Self {
            id: source.id().to_owned(),
            raw_id: Codec::decode_id(source.id())?,
            code: source.code().to_owned(),
            current_weight: source.current_weight(),
            max_capacity: source.max_capacity(),
            status: source.status(),
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

impl Container for ContainerEntity {
    fn id(&self) -> &str {
        &self.id
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn current_weight(&self) -> f64 {
        self.current_weight
    }

    fn max_capacity(&self) -> f64 {
        self.max_capacity
    }

    fn status(&self) -> ContainerStatus {
        self.status
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
