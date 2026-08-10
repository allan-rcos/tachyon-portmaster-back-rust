//! A entity de carga do manifesto.

use sqlx::mysql::MySqlRow;
use sqlx::{FromRow, Row as _};

use crate::entity::codec::Codec;
use chrono::{DateTime, Utc};
use portmaster_domain::domain::ManifestCargo;

/// A entity, com os ids já traduzidos para base62.
pub struct ManifestCargoEntity {
    /// O contêiner, em base62.
    container_id: String,
    /// O produto, em base62.
    product_id: String,
    /// O contêiner como `BIGINT`, para a FK.
    raw_container_id: i64,
    /// O produto como `BIGINT`, para a FK.
    raw_product_id: i64,
    /// Quantas unidades.
    quantity: f64,
    /// O peso correspondente, já pela densidade.
    weight: f64,
    /// Quando a linha entrou no manifesto, em UTC.
    created_at: DateTime<Utc>,
}

impl FromRow<'_, MySqlRow> for ManifestCargoEntity {
    /// Uma linha de `container_items` como a entity a quer.
    fn from_row(row: &MySqlRow) -> sqlx::Result<Self> {
        let raw_container_id: i64 = row.try_get("container_id")?;
        let raw_product_id: i64 = row.try_get("product_id")?;

        Ok(Self {
            container_id: Codec::encode_id(raw_container_id),
            product_id: Codec::encode_id(raw_product_id),
            raw_container_id,
            raw_product_id,
            quantity: row.try_get("quantity")?,
            weight: row.try_get("weight")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

impl ManifestCargoEntity {
    /// Recria a entity a partir de qualquer [`ManifestCargo`], para gravá-la.
    pub(crate) fn from_domain(source: &dyn ManifestCargo) -> anyhow::Result<Self> {
        Ok(Self {
            container_id: source.container_id().to_owned(),
            product_id: source.product_id().to_owned(),
            raw_container_id: Codec::decode_id(source.container_id())?,
            raw_product_id: Codec::decode_id(source.product_id())?,
            quantity: source.quantity(),
            weight: source.weight(),
            created_at: source.created_at(),
        })
    }

    /// O id do contêiner como o banco o guarda.
    pub(crate) const fn raw_container_id(&self) -> i64 {
        self.raw_container_id
    }

    /// O id do produto como o banco o guarda.
    pub(crate) const fn raw_product_id(&self) -> i64 {
        self.raw_product_id
    }
}

impl ManifestCargo for ManifestCargoEntity {
    fn container_id(&self) -> &str {
        &self.container_id
    }

    fn product_id(&self) -> &str {
        &self.product_id
    }

    fn quantity(&self) -> f64 {
        self.quantity
    }

    fn weight(&self) -> f64 {
        self.weight
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
