//! O mock de [`PermissionRepository`].

use mockall::mock;
use portmaster_domain::domain::Permission;
use portmaster_infra::repository::PermissionRepository;

mock! {
    /// O catálogo de permissões, sob controle do teste.
    pub(crate) Permissions {}

    /// O `Clone` que o factory do service exige.
    ///
    /// Nenhum teste clona um mock: quem clona é o controller, uma vez por
    /// requisição, e o bound sobe daí até aqui. Sem `expect_clone`, chamar
    /// `clone` falha — que é o que se quer, porque um mock clonado não
    /// levaria as expectativas do original junto.
    impl Clone for Permissions {
        fn clone(&self) -> Self;
    }

    #[trait_variant::make(Send)]
    impl PermissionRepository for Permissions {
        async fn register(&self, permission: &dyn Permission) -> anyhow::Result<()>;
        async fn all(&self) -> anyhow::Result<Vec<String>>;
        async fn has(&self, slug: &str) -> anyhow::Result<bool>;
    }
}
