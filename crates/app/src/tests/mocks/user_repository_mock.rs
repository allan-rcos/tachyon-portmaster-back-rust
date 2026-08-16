//! O mock de [`UserRepository`].

use mockall::mock;
use portmaster_domain::domain::User;
use portmaster_infra::repository::UserRepository;

mock! {
    /// A persistência de usuários, sob controle do teste.
    pub(crate) Users {}

    /// O `Clone` que o factory do service exige.
    ///
    /// Nenhum teste clona um mock: quem clona é o controller, uma vez por
    /// requisição, e o bound sobe daí até aqui. Sem `expect_clone`, chamar
    /// `clone` falha — que é o que se quer, porque um mock clonado não
    /// levaria as expectativas do original junto.
    impl Clone for Users {
        fn clone(&self) -> Self;
    }

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
