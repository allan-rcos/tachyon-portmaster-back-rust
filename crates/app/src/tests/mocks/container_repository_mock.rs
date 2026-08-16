//! O mock de [`ContainerRepository`].

use mockall::mock;
use portmaster_domain::domain::Container;
use portmaster_infra::repository::ContainerRepository;

mock! {
    /// A persistência de contêineres, sob controle do teste.
    pub(crate) Containers {}

    /// O `Clone` que o factory do service exige.
    ///
    /// Nenhum teste clona um mock: quem clona é o controller, uma vez por
    /// requisição, e o bound sobe daí até aqui. Sem `expect_clone`, chamar
    /// `clone` falha — que é o que se quer, porque um mock clonado não
    /// levaria as expectativas do original junto.
    impl Clone for Containers {
        fn clone(&self) -> Self;
    }

    #[trait_variant::make(Send)]
    impl ContainerRepository for Containers {
        async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn Container>>>;
        async fn insert(&self, container: &dyn Container) -> anyhow::Result<()>;
        async fn update(&self, container: &dyn Container) -> anyhow::Result<()>;
        async fn delete(&self, id: &str) -> anyhow::Result<()>;
    }
}
