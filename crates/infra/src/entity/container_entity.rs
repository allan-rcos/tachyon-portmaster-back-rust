//! A entity de contêiner.

use sqlx::mysql::MySqlRow;
use sqlx::{FromRow, Row as _};

use crate::entity::codec::Codec;
use chrono::{DateTime, Utc};
use portmaster_domain::domain::Container;
use portmaster_domain::enums::ContainerStatus;

/// A entity, com o id já traduzido para base62.
pub struct ContainerEntity {
    /// Identidade em base62.
    id: String,
    /// O mesmo id como `BIGINT`, para os `WHERE` e as FKs.
    ///
    /// Guardado junto do base62 para que a escrita não precise decodificar de
    /// volta a cada consulta.
    raw_id: i64,
    /// O código do contêiner.
    code: String,
    /// Peso embarcado, em quilos.
    current_weight: f64,
    /// Teto de peso.
    max_capacity: f64,
    /// O status, já como enum de domínio — a coluna guarda o índice.
    status: ContainerStatus,
    /// Quando a linha nasceu, em UTC.
    created_at: DateTime<Utc>,
    /// Quando a linha mudou pela última vez, em UTC.
    updated_at: DateTime<Utc>,
    /// Quando foi removida, ou `None` se ativa — o soft-delete.
    deleted_at: Option<DateTime<Utc>>,
}

impl FromRow<'_, MySqlRow> for ContainerEntity {
    /// Uma linha de `containers` como a entity a quer.
    ///
    /// O índice do enum é validado aqui, na leitura: um valor que não
    /// corresponde a variante nenhuma é uma linha que o schema não deveria
    /// admitir, e escolher uma variante por aproximação afirmaria um estado que
    /// o banco não guardou.
    fn from_row(row: &MySqlRow) -> sqlx::Result<Self> {
        let raw_id: i64 = row.try_get("id")?;
        let status: i32 = row.try_get("status")?;

        Ok(Self {
            id: Codec::encode_id(raw_id),
            raw_id,
            code: row.try_get("code")?,
            current_weight: row.try_get("current_weight")?,
            max_capacity: row.try_get("max_capacity")?,
            status: ContainerStatus::from_i32(status).ok_or_else(|| {
                sqlx::Error::Decode(
                    format!("{status} não corresponde a variante nenhuma de ContainerStatus")
                        .into(),
                )
            })?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            deleted_at: row.try_get("deleted_at")?,
        })
    }
}

impl ContainerEntity {
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
