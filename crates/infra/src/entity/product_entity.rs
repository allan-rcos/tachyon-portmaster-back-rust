//! A entity de produto.

use crate::entity::codec::Codec;
use crate::entity::product_row::ProductRow;
use chrono::{DateTime, Utc};
use portmaster_domain::enums::RiskClass;
use portmaster_domain::models::Product;

/// A entity, com o id já traduzido para base62.
pub struct ProductEntity {
    id: String,
    raw_id: i64,
    name: String,
    density: f64,
    risk_class: RiskClass,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl ProductEntity {
    /// Reconstrói a entity a partir de uma linha lida.
    pub(crate) fn from_row(row: ProductRow) -> anyhow::Result<Self> {
        Ok(Self {
            id: Codec::encode_id(row.id),
            raw_id: row.id,
            name: row.name,
            density: row.density,
            risk_class: Codec::decode_enum(row.risk_class, RiskClass::from_i32, "RiskClass")?,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        })
    }

    /// Recria a entity a partir de qualquer [`Product`], para gravá-la.
    pub(crate) fn from_domain(source: &dyn Product) -> anyhow::Result<Self> {
        Ok(Self {
            id: source.id().to_owned(),
            raw_id: Codec::decode_id(source.id())?,
            name: source.name().to_owned(),
            density: source.density(),
            risk_class: source.risk_class(),
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

impl Product for ProductEntity {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn density(&self) -> f64 {
        self.density
    }

    fn risk_class(&self) -> RiskClass {
        self.risk_class
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
