//! A linha crua de `container`, como o `sqlx` a lê.

use chrono::{DateTime, Utc};

/// Uma linha de `containers`.
#[derive(sqlx::FromRow)]
pub(crate) struct ContainerRow {
    /// Snowflake como `BIGINT` — o inteiro não sai desta camada.
    pub(crate) id: i64,
    /// Coluna `code`.
    pub(crate) code: String,
    /// Coluna `current_weight`, em quilos.
    pub(crate) current_weight: f64,
    /// Coluna `max_capacity`, em quilos.
    pub(crate) max_capacity: f64,
    /// Índice da variante de [`ContainerStatus`](portmaster_domain::enums::ContainerStatus), nunca a string.
    pub(crate) status: i32,
    /// Coluna `created_at`, em UTC.
    pub(crate) created_at: DateTime<Utc>,
    /// Coluna `updated_at`, em UTC.
    pub(crate) updated_at: DateTime<Utc>,
    /// Coluna `deleted_at`; `None` é uma linha ativa.
    pub(crate) deleted_at: Option<DateTime<Utc>>,
}
