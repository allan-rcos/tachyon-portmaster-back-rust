//! O mock de [`MarkerGroupRepository`].

use mockall::mock;
use portmaster_domain::domain::MarkerGroup;
use portmaster_infra::repository::MarkerGroupRepository;

mock! {
    /// O catálogo de grupos de marcador, sob controle do teste.
    pub(crate) MarkerGroups {}

    #[trait_variant::make(Send)]
    impl MarkerGroupRepository for MarkerGroups {
        async fn register(&self, group: &dyn MarkerGroup) -> anyhow::Result<()>;
        async fn has(&self, slug: &str) -> anyhow::Result<bool>;
    }
}
