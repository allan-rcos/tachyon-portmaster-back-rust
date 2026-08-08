//! A linha crua de `role`, como o `sqlx` a lê.

use chrono::{DateTime, Utc};

/// Uma linha de `roles`.
#[derive(sqlx::FromRow)]
pub(crate) struct RoleRow {
    /// Snowflake como `BIGINT` — o inteiro não sai desta camada.
    pub(crate) id: i64,
    /// Coluna `name`.
    pub(crate) name: String,
    /// Slugs de permissão, na coluna `JSON`.
    ///
    /// Uma tabela de ligação seria mais ortodoxa, mas a lista é lida inteira a
    /// cada verificação de permissão e nunca é consultada por slug isolado —
    /// então o JOIN não pagaria por si.
    pub(crate) permissions: String,
    /// Coluna `created_at`, em UTC.
    pub(crate) created_at: DateTime<Utc>,
    /// Coluna `updated_at`, em UTC.
    pub(crate) updated_at: DateTime<Utc>,
    /// Coluna `deleted_at`; `None` é uma linha ativa.
    pub(crate) deleted_at: Option<DateTime<Utc>>,
}
