//! O mock de [`ProductRepository`].

use mockall::mock;
use portmaster_domain::domain::Product;
use portmaster_infra::repository::ProductRepository;

mock! {
    /// A persistência de produtos, sob controle do teste.
    pub(crate) Products {}

    /// O `Clone` que o factory do service exige.
    ///
    /// Nenhum teste clona um mock: quem clona é o controller, uma vez por
    /// requisição, e o bound sobe daí até aqui. Sem `expect_clone`, chamar
    /// `clone` falha — que é o que se quer, porque um mock clonado não
    /// levaria as expectativas do original junto.
    impl Clone for Products {
        fn clone(&self) -> Self;
    }

    #[trait_variant::make(Send)]
    impl ProductRepository for Products {
        async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn Product>>>;
        async fn insert(&self, product: &dyn Product) -> anyhow::Result<()>;
        async fn update(&self, product: &dyn Product) -> anyhow::Result<()>;
        async fn delete(&self, id: &str) -> anyhow::Result<()>;
    }
}
