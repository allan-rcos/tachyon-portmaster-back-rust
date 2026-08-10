//! O contrato de persistência de marker group.

use portmaster_domain::domain::MarkerGroup;

/// Registro de grupos de marcador.
#[trait_variant::make(Send)]
pub trait MarkerGroupRepository {
    /// Registra um grupo. Idempotente por slug.
    async fn register(&self, group: &dyn MarkerGroup) -> anyhow::Result<()>;

    /// Se um slug foi registrado.
    async fn has(&self, slug: &str) -> anyhow::Result<bool>;
}
