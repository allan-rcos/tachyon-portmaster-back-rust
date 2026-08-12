//! O mock de [`RoleRepository`].

use mockall::mock;
use portmaster_domain::domain::Role;
use portmaster_infra::repository::RoleRepository;

mock! {
    /// A persistência de papéis, sob controle do teste.
    pub(crate) Roles {}

    #[trait_variant::make(Send)]
    impl RoleRepository for Roles {
        async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn Role>>>;
        async fn find_by_user_id(&self, user_id: &str) -> anyhow::Result<Vec<Box<dyn Role>>>;
        async fn insert(&self, role: &dyn Role) -> anyhow::Result<()>;
        async fn update(&self, role: &dyn Role) -> anyhow::Result<()>;
        async fn delete(&self, id: &str) -> anyhow::Result<()>;
    }
}
