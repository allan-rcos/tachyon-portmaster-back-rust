//! O mock de [`ViewCacheRepository`].

use mockall::mock;
use portmaster_infra::repository::ViewCacheRepository;
use serde::de::DeserializeOwned;
use serde::Serialize;

mock! {
    /// O cache de leitura, sob controle do teste.
    ///
    /// `get` e `put` são genéricos sobre a View, então a expectativa se arma
    /// nomeando o tipo concreto: `expect_get::<RoleListView>()`.
    pub(crate) ViewCache {}

    #[trait_variant::make(Send)]
    impl ViewCacheRepository for ViewCache {
        async fn get<V: DeserializeOwned + 'static>(&self, group: &str, key: &str)
            -> anyhow::Result<Option<V>>;

        async fn put<V: Serialize + Sync + 'static>(&self, group: &str, key: &str, view: &V)
            -> anyhow::Result<()>;

        async fn invalidate(&self, group: &str) -> anyhow::Result<()>;
    }
}
