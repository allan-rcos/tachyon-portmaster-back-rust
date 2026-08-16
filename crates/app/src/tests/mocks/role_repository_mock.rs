//! O mock de [`RoleRepository`].

use mockall::mock;
use portmaster_domain::domain::Role;
use portmaster_infra::repository::RoleRepository;

mock! {
    /// A persistência de papéis, sob controle do teste.
    pub(crate) Roles {}

    /// O `Clone` que o factory do service exige.
    ///
    /// Nenhum teste clona um mock: quem clona é o controller, uma vez por
    /// requisição, e o bound sobe daí até aqui. Sem `expect_clone`, chamar
    /// `clone` falha — que é o que se quer, porque um mock clonado não
    /// levaria as expectativas do original junto.
    impl Clone for Roles {
        fn clone(&self) -> Self;
    }

    #[trait_variant::make(Send)]
    impl RoleRepository for Roles {
        async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn Role>>>;
        async fn find_by_user_id(&self, user_id: &str) -> anyhow::Result<Vec<Box<dyn Role>>>;
        async fn insert(&self, role: &dyn Role) -> anyhow::Result<()>;
        async fn update(&self, role: &dyn Role) -> anyhow::Result<()>;
        async fn delete(&self, id: &str) -> anyhow::Result<()>;
    }
}
