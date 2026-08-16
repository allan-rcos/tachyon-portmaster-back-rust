//! A entity de contêiner.

use chrono::{DateTime, Utc};
use mysql_async::prelude::FromRow;
use portmaster_domain::domain::Container;
use portmaster_domain::enums::ContainerStatus;

use crate::entity::decode::Decode;
use crate::entity::entity_id::EntityId;

/// A entity, que é também a linha de `containers`.
#[derive(Clone, FromRow)]
pub struct ContainerEntity {
    /// A identidade, nas duas formas.
    #[mysql(
        deserialize_with = "Decode::entity_id",
        serialize_with = "Decode::entity_id_value"
    )]
    id: EntityId,
    /// O código do contêiner.
    code: String,
    /// Peso embarcado, em quilos.
    current_weight: f64,
    /// Teto de peso.
    max_capacity: f64,
    /// O status, já como enum de domínio — a coluna guarda o índice.
    ///
    /// O [`Decode`] valida o índice na leitura: um valor que não corresponde a
    /// variante nenhuma é uma linha que o schema não deveria admitir, e escolher
    /// uma variante por aproximação afirmaria um estado que o banco não guardou.
    #[mysql(
        deserialize_with = "Decode::container_status",
        serialize_with = "Decode::container_status_value"
    )]
    status: ContainerStatus,
    /// Quando a linha nasceu, em UTC.
    #[mysql(deserialize_with = "Decode::utc", serialize_with = "Decode::utc_value")]
    created_at: DateTime<Utc>,
    /// Quando a linha mudou pela última vez, em UTC.
    #[mysql(deserialize_with = "Decode::utc", serialize_with = "Decode::utc_value")]
    updated_at: DateTime<Utc>,
    /// Quando foi removida, ou `None` se ativa — o soft-delete.
    #[mysql(deserialize_with = "Decode::utc_opt")]
    deleted_at: Option<DateTime<Utc>>,
}

impl ContainerEntity {
    /// Recria a entity a partir de qualquer [`Container`], para gravá-la.
    pub(crate) fn from_domain(source: &dyn Container) -> anyhow::Result<Self> {
        Ok(Self {
            id: EntityId::try_from(source.id())?,
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
        self.id.raw()
    }
}

impl Container for ContainerEntity {
    fn id(&self) -> &str {
        self.id.as_str()
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
