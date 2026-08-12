//! O mock de [`ContainerRepository`].

use mockall::mock;
use portmaster_domain::domain::Container;
use portmaster_infra::repository::ContainerRepository;

mock! {
    /// A persistência de contêineres, sob controle do teste.
    pub(crate) Containers {}

    #[trait_variant::make(Send)]
    impl ContainerRepository for Containers {
        async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn Container>>>;
        async fn insert(&self, container: &dyn Container) -> anyhow::Result<()>;
        async fn update(&self, container: &dyn Container) -> anyhow::Result<()>;
        async fn delete(&self, id: &str) -> anyhow::Result<()>;
    }
}
