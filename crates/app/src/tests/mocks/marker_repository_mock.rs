//! O mock de [`MarkerRepository`].

use mockall::mock;
use portmaster_domain::domain::Marker;
use portmaster_infra::repository::MarkerRepository;

mock! {
    /// A persistência de marcadores, sob controle do teste.
    pub(crate) Markers {}

    #[trait_variant::make(Send)]
    impl MarkerRepository for Markers {
        async fn put(&self, marker: &dyn Marker, ttl_seconds: u64) -> anyhow::Result<()>;
        async fn is_valid(&self, group: &str, key: &str) -> anyhow::Result<bool>;
    }
}
