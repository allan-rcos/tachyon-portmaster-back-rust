//! A linha crua de `manifest_cargo`, como o `sqlx` a lê.

use chrono::{DateTime, Utc};

/// Uma linha de `container_items`.
///
/// Entidade fraca: só `created_at`, sem `updated_at` nem `deleted_at`. Não é
/// atualizada nem sofre soft-delete — mudar uma linha é removê-la e recriá-la,
/// e removê-la é `DELETE` de verdade.
#[derive(sqlx::FromRow)]
pub(crate) struct ManifestCargoRow {
    /// FK do contêiner, como `BIGINT`.
    pub(crate) container_id: i64,
    /// FK do produto, como `BIGINT`.
    pub(crate) product_id: i64,
    /// Coluna `quantity`.
    pub(crate) quantity: f64,
    /// Coluna `weight`, já calculada na escrita.
    pub(crate) weight: f64,
    /// Quando a linha entrou, em UTC.
    pub(crate) created_at: DateTime<Utc>,
}
