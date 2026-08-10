//! A entity de produto.

use chrono::{DateTime, Utc};
use portmaster_domain::domain::Product;
use portmaster_domain::enums::RiskClass;
use sqlx::mysql::MySqlRow;
use sqlx::{FromRow, Row as _};

use crate::entity::codec::Codec;

/// A entity, com o id já traduzido para base62.
///
/// É ela quem implementa [`FromRow`]: a linha crua não vira uma struct própria
/// antes de virar entity, porque essa struct não tinha comportamento nenhum —
/// era a mesma lista de colunas escrita duas vezes, e a segunda existia só para
/// ser convertida na primeira.
pub struct ProductEntity {
    /// Identidade em base62.
    id: String,
    /// O mesmo id como `BIGINT`, para os `WHERE` e as FKs.
    ///
    /// Guardado junto do base62 para que a escrita não precise decodificar de
    /// volta a cada consulta.
    raw_id: i64,
    /// Nome comercial.
    name: String,
    /// Quilos por litro.
    density: f64,
    /// A classe de risco, já como enum de domínio.
    risk_class: RiskClass,
    /// Quando a linha nasceu, em UTC.
    created_at: DateTime<Utc>,
    /// Quando a linha mudou pela última vez, em UTC.
    updated_at: DateTime<Utc>,
    /// Quando foi removida, ou `None` se ativa — o soft-delete.
    deleted_at: Option<DateTime<Utc>>,
}

impl ProductEntity {
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

impl FromRow<'_, MySqlRow> for ProductEntity {
    /// Uma linha de `products` como a entity a quer.
    ///
    /// O índice do enum é validado aqui, na leitura: um valor que não
    /// corresponde a variante nenhuma é uma linha que o schema não deveria
    /// admitir, e escolher uma variante por aproximação afirmaria uma classe de
    /// risco que o banco não guardou.
    fn from_row(row: &MySqlRow) -> sqlx::Result<Self> {
        let raw_id: i64 = row.try_get("id")?;
        let risk_class: i32 = row.try_get("risk_class")?;

        Ok(Self {
            id: Codec::encode_id(raw_id),
            raw_id,
            name: row.try_get("name")?,
            density: row.try_get("density")?,
            risk_class: RiskClass::from_i32(risk_class).ok_or_else(|| {
                sqlx::Error::Decode(
                    format!("{risk_class} não corresponde a variante nenhuma de RiskClass").into(),
                )
            })?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            deleted_at: row.try_get("deleted_at")?,
        })
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
