//! Os testes de `marker_repository`.

use super::*;
use crate::repository::memory::marker_group_repository::MarkerGroupMemoryRepository;
use crate::scope::memory::intern::moka_cache::MokaCache;
use crate::scope::memory::intern::moka_store::MokaStore;
use portmaster_domain::domain::MarkerGroup;

/// Marcador mínimo, sem passar pelo `TableModule`.
struct StubMarker {
    group: &'static str,
    key: &'static str,
    flag: bool,
}

impl Marker for StubMarker {
    fn group(&self) -> &str {
        self.group
    }
    fn key(&self) -> &str {
        self.key
    }
    fn flag(&self) -> bool {
        self.flag
    }
}

struct StubGroup(&'static str);
impl MarkerGroup for StubGroup {
    fn slug(&self) -> &str {
        self.0
    }
}

/// Repositório com o grupo `refresh-token` já registrado.
async fn repository() -> MarkerMemoryRepository<MokaStore, MarkerGroupMemoryRepository<MokaStore>> {
    let metadata = MokaStore::new("marker-group", MokaCache::new(100));
    let groups = MarkerGroupMemoryRepository::new(metadata.clone());
    groups.register(&StubGroup("refresh-token")).await.unwrap();

    MarkerMemoryRepository::new(
        MokaStore::new("marker", MokaCache::new(100)),
        MarkerGroupMemoryRepository::new(metadata),
    )
}

fn marker(flag: bool) -> StubMarker {
    StubMarker {
        group: "refresh-token",
        key: "abc123",
        flag,
    }
}

#[tokio::test]
async fn marcar_e_consultar() {
    let repository = repository().await;
    repository.put(&marker(true), 60).await.unwrap();

    assert!(repository
        .is_valid("refresh-token", "abc123")
        .await
        .unwrap());
}

#[tokio::test]
async fn marca_inexistente_nao_e_valida() {
    let repository = repository().await;
    assert!(!repository
        .is_valid("refresh-token", "nunca-visto")
        .await
        .unwrap());
}

#[tokio::test]
async fn invalidar_e_permitido_e_idempotente() {
    // O caminho do logout, e do logout chamado duas vezes.
    let repository = repository().await;
    repository.put(&marker(true), 60).await.unwrap();
    repository.put(&marker(false), 60).await.unwrap();
    repository.put(&marker(false), 60).await.unwrap();

    assert!(!repository
        .is_valid("refresh-token", "abc123")
        .await
        .unwrap());
}

/// Se isto passasse, um logout seria reversível por quem guardasse o token
/// antigo.
#[tokio::test]
async fn nao_revalida_marca_invalidada() {
    let repository = repository().await;
    repository.put(&marker(true), 60).await.unwrap();
    repository.put(&marker(false), 60).await.unwrap();

    assert!(repository.put(&marker(true), 60).await.is_err());
}

#[tokio::test]
async fn nao_remarca_o_que_ja_esta_valido() {
    let repository = repository().await;
    repository.put(&marker(true), 60).await.unwrap();

    assert!(repository.put(&marker(true), 60).await.is_err());
}

/// Um slug com erro de digitação criaria um espaço de nomes paralelo em que
/// nada seria reencontrado.
#[tokio::test]
async fn recusa_grupo_nao_registrado() {
    let repository = repository().await;
    let stray = StubMarker {
        group: "refresh-tokens",
        key: "abc",
        flag: true,
    };

    assert!(repository.put(&stray, 60).await.is_err());
}

/// O marcador e o grupo vivem em stores distintos: se dividissem o mesmo,
/// um grupo declarado responderia como marcador válido.
#[tokio::test]
async fn grupo_declarado_nao_e_marcador_valido() {
    let repository = repository().await;

    assert!(!repository
        .is_valid("marker-group", "refresh-token")
        .await
        .unwrap());
}
