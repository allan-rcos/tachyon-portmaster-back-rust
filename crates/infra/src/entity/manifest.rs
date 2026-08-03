//! A entity de carga do manifesto.

use chrono::{DateTime, Utc};
use portmaster_domain::manifest::ManifestCargo;

use super::{decode_id, encode_id};

/// Uma linha de `container_items`.
///
/// Entidade fraca: só `created_at`, sem `updated_at` nem `deleted_at`. Não é
/// atualizada nem sofre soft-delete — mudar uma linha é removê-la e recriá-la,
/// e removê-la é `DELETE` de verdade.
#[derive(sqlx::FromRow)]
pub(crate) struct ManifestCargoRow {
    pub(crate) container_id: i64,
    pub(crate) product_id: i64,
    pub(crate) quantity: f64,
    pub(crate) weight: f64,
    pub(crate) created_at: DateTime<Utc>,
}

/// A entity, com os ids já traduzidos para base62.
pub(crate) struct ManifestCargoEntity {
    container_id: String,
    product_id: String,
    raw_container_id: i64,
    raw_product_id: i64,
    quantity: f64,
    weight: f64,
    created_at: DateTime<Utc>,
}

impl ManifestCargoEntity {
    /// Reconstrói a entity a partir de uma linha lida.
    pub(crate) fn from_row(row: ManifestCargoRow) -> Self {
        Self {
            container_id: encode_id(row.container_id),
            product_id: encode_id(row.product_id),
            raw_container_id: row.container_id,
            raw_product_id: row.product_id,
            quantity: row.quantity,
            weight: row.weight,
            created_at: row.created_at,
        }
    }

    /// Recria a entity a partir de qualquer [`ManifestCargo`], para gravá-la.
    pub(crate) fn from_domain(source: &dyn ManifestCargo) -> anyhow::Result<Self> {
        Ok(Self {
            container_id: source.container_id().to_owned(),
            product_id: source.product_id().to_owned(),
            raw_container_id: decode_id(source.container_id())?,
            raw_product_id: decode_id(source.product_id())?,
            quantity: source.quantity(),
            weight: source.weight(),
            created_at: source.created_at(),
        })
    }

    /// O id do contêiner como o banco o guarda.
    pub(crate) fn raw_container_id(&self) -> i64 {
        self.raw_container_id
    }

    /// O id do produto como o banco o guarda.
    pub(crate) fn raw_product_id(&self) -> i64 {
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
