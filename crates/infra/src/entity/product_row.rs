//! A linha crua de `product`, como o `sqlx` a lê.

use chrono::{DateTime, Utc};

/// Uma linha de `products`.
#[derive(sqlx::FromRow)]
pub(crate) struct ProductRow {
    /// Snowflake como `BIGINT` — o inteiro não sai desta camada.
    pub(crate) id: i64,
    /// Coluna `name`.
    pub(crate) name: String,
    /// Coluna `density`, em quilos por litro.
    pub(crate) density: f64,
    /// Índice da variante de [`RiskClass`](portmaster_domain::enums::RiskClass), nunca a string.
    pub(crate) risk_class: i32,
    /// Coluna `created_at`, em UTC.
    pub(crate) created_at: DateTime<Utc>,
    /// Coluna `updated_at`, em UTC.
    pub(crate) updated_at: DateTime<Utc>,
    /// Coluna `deleted_at`; `None` é uma linha ativa.
    pub(crate) deleted_at: Option<DateTime<Utc>>,
}
