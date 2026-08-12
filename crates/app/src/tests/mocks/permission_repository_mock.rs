//! O mock de [`PermissionRepository`].

use mockall::mock;
use portmaster_domain::domain::Permission;
use portmaster_infra::repository::PermissionRepository;

mock! {
    /// O catálogo de permissões, sob controle do teste.
    pub(crate) Permissions {}

    #[trait_variant::make(Send)]
    impl PermissionRepository for Permissions {
        async fn register(&self, permission: &dyn Permission) -> anyhow::Result<()>;
        async fn all(&self) -> anyhow::Result<Vec<String>>;
        async fn has(&self, slug: &str) -> anyhow::Result<bool>;
    }
}
