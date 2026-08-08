//! A linha crua de `user`, como o `sqlx` a lê.

use chrono::{DateTime, Utc};

/// Uma linha de `users`.
#[derive(sqlx::FromRow)]
pub(crate) struct UserRow {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) password_hash: String,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) deleted_at: Option<DateTime<Utc>>,
}
