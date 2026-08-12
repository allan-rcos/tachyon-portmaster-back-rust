//! A entity de produto.

use chrono::{DateTime, Utc};
use portmaster_domain::domain::Product;
use portmaster_domain::enums::RiskClass;
use sqlx::FromRow;

use crate::entity::entity_id::EntityId;

/// A entity, que é também a linha de `products`.
#[derive(Clone, FromRow)]
pub struct ProductEntity {
    /// A identidade, nas duas formas.
    #[sqlx(try_from = "i64")]
    id: EntityId,
    /// Nome comercial.
    name: String,
    /// Quilos por litro.
    density: f64,
    /// A classe de risco, já como enum de domínio — a coluna guarda o índice.
    ///
    /// O `try_from` valida o índice na leitura: um valor que não corresponde a
    /// variante nenhuma afirmaria uma classe de risco que o banco não guardou.
    #[sqlx(try_from = "i32")]
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
            id: EntityId::try_from(source.id())?,
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
        self.id.raw()
    }
}

impl Product for ProductEntity {
    fn id(&self) -> &str {
        self.id.as_str()
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
