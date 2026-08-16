//! O mock de [`MarkerGroupRepository`].

use mockall::mock;
use portmaster_domain::domain::MarkerGroup;
use portmaster_infra::repository::MarkerGroupRepository;

mock! {
    /// O catálogo de grupos de marcador, sob controle do teste.
    pub(crate) MarkerGroups {}

    /// O `Clone` que o factory do service exige.
    ///
    /// Nenhum teste clona um mock: quem clona é o controller, uma vez por
    /// requisição, e o bound sobe daí até aqui. Sem `expect_clone`, chamar
    /// `clone` falha — que é o que se quer, porque um mock clonado não
    /// levaria as expectativas do original junto.
    impl Clone for MarkerGroups {
        fn clone(&self) -> Self;
    }

    #[trait_variant::make(Send)]
    impl MarkerGroupRepository for MarkerGroups {
        async fn register(&self, group: &dyn MarkerGroup) -> anyhow::Result<()>;
        async fn has(&self, slug: &str) -> anyhow::Result<bool>;
    }
}
