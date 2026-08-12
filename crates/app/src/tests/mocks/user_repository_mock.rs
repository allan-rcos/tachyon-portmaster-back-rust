//! O mock de [`UserRepository`].

use mockall::mock;
use portmaster_domain::domain::User;
use portmaster_infra::repository::UserRepository;

mock! {
    /// A persistência de usuários, sob controle do teste.
    pub(crate) Users {}

    #[trait_variant::make(Send)]
    impl UserRepository for Users {
        async fn has_any(&self) -> anyhow::Result<bool>;
        async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn User>>>;
        async fn find_by_email(&self, email: &str) -> anyhow::Result<Option<Box<dyn User>>>;
        async fn insert(&self, user: &dyn User) -> anyhow::Result<()>;
        async fn update(&self, user: &dyn User) -> anyhow::Result<()>;
        async fn delete(&self, id: &str) -> anyhow::Result<()>;
        async fn sync_roles(&self, user_id: &str, role_ids: &[String]) -> anyhow::Result<()>;
    }
}
