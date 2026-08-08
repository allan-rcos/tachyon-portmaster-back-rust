//! A linha crua de `user`, como o `sqlx` a lê.

use chrono::{DateTime, Utc};

/// Uma linha de `users`.
#[derive(sqlx::FromRow)]
pub(crate) struct UserRow {
    /// Snowflake como `BIGINT` — o inteiro não sai desta camada.
    pub(crate) id: i64,
    /// Coluna `name`.
    pub(crate) name: String,
    /// Coluna `email`, única entre os não-removidos.
    pub(crate) email: String,
    /// Coluna `password_hash`, no formato PHC do Argon2.
    pub(crate) password_hash: String,
    /// Coluna `created_at`, em UTC.
    pub(crate) created_at: DateTime<Utc>,
    /// Coluna `updated_at`, em UTC.
    pub(crate) updated_at: DateTime<Utc>,
    /// Coluna `deleted_at`; `None` é uma linha ativa.
    pub(crate) deleted_at: Option<DateTime<Utc>>,
}
