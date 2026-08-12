//! O mock de [`ManifestRepository`].

#![allow(
    clippy::ref_option_ref,
    reason = "o predicado que o mock! gera recebe &Option<&T>; a assinatura é do macro, não deste arquivo"
)]

use mockall::mock;
use portmaster_domain::domain::ManifestCargo;
use portmaster_domain::enums::TelemetryEvent;
use portmaster_infra::repository::ManifestRepository;

mock! {
    /// A persistência do manifesto, sob controle do teste.
    pub(crate) Manifests {}

    #[trait_variant::make(Send)]
    impl ManifestRepository for Manifests {
        async fn find_cargo(&self, container_id: &str, product_id: &str)
            -> anyhow::Result<Option<Box<dyn ManifestCargo>>>;
        async fn upsert_cargo(&self, cargo: &dyn ManifestCargo) -> anyhow::Result<()>;
        async fn delete_cargo(&self, container_id: &str, product_id: &str) -> anyhow::Result<()>;
        async fn clear_manifest(&self, container_id: &str) -> anyhow::Result<()>;
        async fn insert_telemetry<'a>(&self, container_id: &str, event: TelemetryEvent,
            description: Option<&'a str>) -> anyhow::Result<()>;
    }
}
