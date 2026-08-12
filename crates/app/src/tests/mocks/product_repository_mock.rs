//! O mock de [`ProductRepository`].

use mockall::mock;
use portmaster_domain::domain::Product;
use portmaster_infra::repository::ProductRepository;

mock! {
    /// A persistência de produtos, sob controle do teste.
    pub(crate) Products {}

    #[trait_variant::make(Send)]
    impl ProductRepository for Products {
        async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Box<dyn Product>>>;
        async fn insert(&self, product: &dyn Product) -> anyhow::Result<()>;
        async fn update(&self, product: &dyn Product) -> anyhow::Result<()>;
        async fn delete(&self, id: &str) -> anyhow::Result<()>;
    }
}
