//! Os testes de `permission_repository`.

use super::*;
use crate::scope::memory::intern::moka_cache::MokaCache;
use crate::scope::memory::intern::moka_store::MokaStore;
use pretty_assertions::assert_eq;

/// Permissão mínima, sem passar pelo `TableModule`.
struct StubPermission(&'static str);
impl Permission for StubPermission {
    fn slug(&self) -> &str {
        self.0
    }
}

fn repository() -> PermissionMemoryRepository<MokaStore> {
    PermissionMemoryRepository::new(MokaStore::new("permission", MokaCache::new(100)))
}

/// Cada caso de uso declara a sua permissão ao ser construído, e o provider
/// pode construí-lo mais de uma vez.
#[tokio::test]
async fn registrar_a_mesma_permissao_duas_vezes_nao_duplica() {
    let repository = repository();

    repository
        .register(&StubPermission("product:create"))
        .await
        .unwrap();
    repository
        .register(&StubPermission("product:create"))
        .await
        .unwrap();

    assert_eq!(repository.all().await.unwrap(), vec!["product:create"]);
}

#[tokio::test]
async fn a_listagem_sai_ordenada() {
    let repository = repository();

    for slug in ["user:list", "container:seal", "product:create"] {
        repository.register(&StubPermission(slug)).await.unwrap();
    }

    assert_eq!(
        repository.all().await.unwrap(),
        vec!["container:seal", "product:create", "user:list"]
    );
}

#[tokio::test]
async fn has_responde_pelo_que_foi_registrado() {
    let repository = repository();
    repository
        .register(&StubPermission("metrics:read"))
        .await
        .unwrap();

    assert!(repository.has("metrics:read").await.unwrap());
    assert!(!repository.has("metrics:write").await.unwrap());
}

/// O boot roda fora de qualquer escopo, e precisa continuar valendo na hora.
#[tokio::test]
async fn escrita_fora_do_escopo_vale_na_hora() {
    let repository = repository();
    repository
        .register(&StubPermission("metrics:read"))
        .await
        .unwrap();

    assert!(!crate::scope::MasterScope::is_active());
    assert!(repository.has("metrics:read").await.unwrap());
}
