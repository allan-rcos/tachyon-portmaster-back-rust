//! O contrato de persistência de manifest.

use portmaster_domain::domain::ManifestCargo;
use portmaster_domain::enums::TelemetryEvent;

/// Persistência de carga e telemetria.
#[trait_variant::make(Send)]
pub trait ManifestRepository {
    /// A linha de manifesto de um produto num contêiner, se existe.
    async fn find_cargo(
        &self,
        container_id: &str,
        product_id: &str,
    ) -> anyhow::Result<Option<Box<dyn ManifestCargo>>>;

    /// Grava ou substitui a linha de manifesto.
    async fn upsert_cargo(&self, cargo: &dyn ManifestCargo) -> anyhow::Result<()>;

    /// Apaga a linha de um produto.
    ///
    /// `DELETE` de verdade: carga é entidade fraca, sem soft-delete.
    async fn delete_cargo(&self, container_id: &str, product_id: &str) -> anyhow::Result<()>;

    /// Apaga o manifesto inteiro de um contêiner.
    async fn clear_manifest(&self, container_id: &str) -> anyhow::Result<()>;

    /// Registra um movimento na telemetria.
    async fn insert_telemetry(
        &self,
        container_id: &str,
        event: TelemetryEvent,
        description: Option<&str>,
    ) -> anyhow::Result<()>;
}
