//! O mock de [`MarkerRepository`].

use mockall::mock;
use portmaster_domain::domain::Marker;
use portmaster_infra::repository::MarkerRepository;

mock! {
    /// A persistência de marcadores, sob controle do teste.
    pub(crate) Markers {}

    /// O `Clone` que o factory do service exige.
    ///
    /// Nenhum teste clona um mock: quem clona é o controller, uma vez por
    /// requisição, e o bound sobe daí até aqui. Sem `expect_clone`, chamar
    /// `clone` falha — que é o que se quer, porque um mock clonado não
    /// levaria as expectativas do original junto.
    impl Clone for Markers {
        fn clone(&self) -> Self;
    }

    #[trait_variant::make(Send)]
    impl MarkerRepository for Markers {
        async fn put(&self, marker: &dyn Marker, ttl_seconds: u64) -> anyhow::Result<()>;
        async fn is_valid(&self, group: &str, key: &str) -> anyhow::Result<bool>;
    }
}
