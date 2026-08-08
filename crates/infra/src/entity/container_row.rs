//! A linha crua de `container`, como o `sqlx` a lê.

use chrono::{DateTime, Utc};

/// Uma linha de `containers`.
#[derive(sqlx::FromRow)]
pub(crate) struct ContainerRow {
    pub(crate) id: i64,
    pub(crate) code: String,
    pub(crate) current_weight: f64,
    pub(crate) max_capacity: f64,
    /// Índice da variante de [`ContainerStatus`](portmaster_domain::enums::ContainerStatus), nunca a string.
    pub(crate) status: i32,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) deleted_at: Option<DateTime<Utc>>,
}
