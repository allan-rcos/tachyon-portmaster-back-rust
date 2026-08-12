//! Os testes de `marker_group_repository`.

use super::*;
use crate::scope::memory::intern::moka_cache::MokaCache;
use crate::scope::memory::intern::moka_store::MokaStore;

/// Grupo mínimo, sem passar pelo `TableModule`.
struct StubGroup(&'static str);
impl MarkerGroup for StubGroup {
    fn slug(&self) -> &str {
        self.0
    }
}

fn repository() -> MarkerGroupMemoryRepository<MokaStore> {
    MarkerGroupMemoryRepository::new(MokaStore::new("marker-group", MokaCache::new(100)))
}

#[tokio::test]
async fn registrar_e_consultar() {
    let repository = repository();
    repository
        .register(&StubGroup("refresh-token"))
        .await
        .unwrap();

    assert!(repository.has("refresh-token").await.unwrap());
    assert!(!repository.has("nunca-declarado").await.unwrap());
}

#[tokio::test]
async fn registrar_duas_vezes_nao_e_erro() {
    let repository = repository();
    repository
        .register(&StubGroup("refresh-token"))
        .await
        .unwrap();
    repository
        .register(&StubGroup("refresh-token"))
        .await
        .unwrap();

    assert!(repository.has("refresh-token").await.unwrap());
}
