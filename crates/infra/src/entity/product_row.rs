//! A linha crua de `product`, como o `sqlx` a lê.

use chrono::{DateTime, Utc};

/// Uma linha de `products`.
#[derive(sqlx::FromRow)]
pub(crate) struct ProductRow {
    /// Snowflake como `BIGINT` — o inteiro não sai desta camada.
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) density: f64,
    /// Índice da variante de [`RiskClass`](portmaster_domain::enums::RiskClass), nunca a string.
    pub(crate) risk_class: i32,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) deleted_at: Option<DateTime<Utc>>,
}
