//! A entity de carga do manifesto.

use chrono::{DateTime, Utc};
use mysql_async::prelude::FromRow;
use portmaster_domain::domain::ManifestCargo;

use crate::entity::decode::Decode;
use crate::entity::entity_id::EntityId;

/// A entity, que é também a linha de `container_items`.
///
/// Dois ids, e não quatro campos: cada [`EntityId`] carrega o `BIGINT` da FK e o
/// base62 que o domínio expõe.
#[derive(Clone, FromRow)]
pub struct ManifestCargoEntity {
    /// O contêiner, nas duas formas.
    #[mysql(
        deserialize_with = "Decode::entity_id",
        serialize_with = "Decode::entity_id_value"
    )]
    container_id: EntityId,
    /// O produto, nas duas formas.
    #[mysql(
        deserialize_with = "Decode::entity_id",
        serialize_with = "Decode::entity_id_value"
    )]
    product_id: EntityId,
    /// Quantas unidades.
    quantity: f64,
    /// O peso correspondente, já pela densidade.
    weight: f64,
    /// Quando a linha entrou no manifesto, em UTC.
    #[mysql(deserialize_with = "Decode::utc", serialize_with = "Decode::utc_value")]
    created_at: DateTime<Utc>,
}

impl ManifestCargoEntity {
    /// Recria a entity a partir de qualquer [`ManifestCargo`], para gravá-la.
    pub(crate) fn from_domain(source: &dyn ManifestCargo) -> anyhow::Result<Self> {
        Ok(Self {
            container_id: EntityId::try_from(source.container_id())?,
            product_id: EntityId::try_from(source.product_id())?,
            quantity: source.quantity(),
            weight: source.weight(),
            created_at: source.created_at(),
        })
    }

    /// O id do contêiner como o banco o guarda.
    pub(crate) const fn raw_container_id(&self) -> i64 {
        self.container_id.raw()
    }

    /// O id do produto como o banco o guarda.
    pub(crate) const fn raw_product_id(&self) -> i64 {
        self.product_id.raw()
    }
}

impl ManifestCargo for ManifestCargoEntity {
    fn container_id(&self) -> &str {
        self.container_id.as_str()
    }

    fn product_id(&self) -> &str {
        self.product_id.as_str()
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
