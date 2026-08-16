//! Os testes de `manifest_service_impl`.
//!
//! O `persist` tem três ramos exclusivos — limpar o manifesto, apagar a linha,
//! gravá-la — e quem os decide é o `ManifestChange` que o domínio devolveu.
//! Metade deste arquivo existe para afirmar que o service **obedece** a essa
//! decisão em vez de tomá-la de novo: reimplementá-la aqui daria ao sistema duas
//! opiniões sobre a mesma coisa.

use portmaster_domain::enums::ContainerStatus;
use portmaster_domain::error::ManifestError as DomainManifestError;

use super::*;
use crate::tests::factories::container_factory::StubContainer;
use crate::tests::factories::manifest_cargo_factory::StubCargo;
use crate::tests::factories::manifest_change_factory::StubManifestChange;
use crate::tests::factories::product_factory::StubProduct;
use crate::tests::factories::user_context_factory::user_with;
use crate::tests::mocks::container_repository_mock::MockContainers;
use crate::tests::mocks::manifest_repository_mock::MockManifests;
use crate::tests::mocks::manifest_tm_mock::MockManifestRules;
use crate::tests::mocks::product_repository_mock::MockProducts;
use crate::tests::mocks::view_cache_repository_mock::MockViewCache;

/// O service com os mocks que o teste armou.
fn service(
    containers: MockContainers,
    products: MockProducts,
    manifests: MockManifests,
    manifest_rules: MockManifestRules,
    views: MockViewCache,
) -> impl ManifestService {
    manifest_service(containers, products, manifests, manifest_rules, views)
}

/// O comando de embarque.
fn command(context: crate::context::UserContext) -> MoveItemCommand {
    MoveItemCommand {
        context,
        container_id: "9Z8y".to_owned(),
        product_id: "aZl8".to_owned(),
        quantity: 10.0,
    }
}

/// Um contêiner e um produto que existem, para os testes do caminho adiante.
fn found(containers: &mut MockContainers, products: &mut MockProducts) {
    containers
        .expect_find_by_id()
        .times(1)
        .returning(|id| Ok(Some(StubContainer::boxed(id, ContainerStatus::Loading))));
    products
        .expect_find_by_id()
        .times(1)
        .returning(|id| Ok(Some(StubProduct::boxed(id, 0.75))));
}

/// Sem a permissão, a recusa acontece antes de qualquer port ser tocada.
#[tokio::test]
async fn embarcar_sem_permissao_nao_toca_em_port_nenhuma() {
    let mut containers = MockContainers::new();
    containers.expect_find_by_id().never();

    let Err(error) = service(
        containers,
        MockProducts::new(),
        MockManifests::new(),
        MockManifestRules::new(),
        MockViewCache::new(),
    )
    .load(command(user_with(&[])))
    .await
    else {
        panic!("sem a permissão, embarcar tem de recusar");
    };

    assert!(matches!(
        error,
        ManifestError::App(AppError::PermissionDenied {
            permission: "manifest:load"
        })
    ));
}

/// Contêiner inexistente recusa antes de o produto ser procurado.
#[tokio::test]
async fn conteiner_inexistente_recusa_antes_do_produto() {
    let mut containers = MockContainers::new();
    containers
        .expect_find_by_id()
        .times(1)
        .returning(|_| Ok(None));

    let mut products = MockProducts::new();
    products.expect_find_by_id().never();

    let Err(error) = service(
        containers,
        products,
        MockManifests::new(),
        MockManifestRules::new(),
        MockViewCache::new(),
    )
    .load(command(user_with(&["manifest:load"])))
    .await
    else {
        panic!("contêiner inexistente recusa");
    };

    assert!(matches!(error, ManifestError::MissingContainer(id) if id == "9Z8y"));
}

/// A operação que o domínio recusa não grava nada nem derruba o cache.
#[tokio::test]
async fn operacao_recusada_nao_grava_nem_derruba_o_cache() {
    let mut containers = MockContainers::new();
    let mut products = MockProducts::new();
    found(&mut containers, &mut products);
    containers.expect_update().never();

    let mut manifests = MockManifests::new();
    manifests
        .expect_find_cargo()
        .times(1)
        .returning(|_, _| Ok(None));
    manifests.expect_upsert_cargo().never();
    manifests.expect_insert_telemetry().never();

    let mut manifest_rules = MockManifestRules::new();
    manifest_rules
        .expect_load()
        .times(1)
        .returning(|_, _, _, _| Err(DomainManifestError::ContainerClosed));

    let mut views = MockViewCache::new();
    views.expect_invalidate().never();

    let Err(error) = service(containers, products, manifests, manifest_rules, views)
        .load(command(user_with(&["manifest:load"])))
        .await
    else {
        panic!("a operação recusada não passa");
    };

    assert!(matches!(error, ManifestError::Refused(_)));
}

/// Quando o domínio manda gravar a linha, é `upsert_cargo` que acontece.
#[tokio::test]
async fn mudanca_com_carga_grava_a_linha() {
    let mut containers = MockContainers::new();
    let mut products = MockProducts::new();
    found(&mut containers, &mut products);
    containers.expect_update().times(1).returning(|_| Ok(()));

    let mut manifests = MockManifests::new();
    manifests
        .expect_find_cargo()
        .times(1)
        .returning(|_, _| Ok(None));
    manifests
        .expect_upsert_cargo()
        .withf(|cargo| cargo.quantity() == 10.0)
        .times(1)
        .returning(|_| Ok(()));
    manifests.expect_clear_manifest().never();
    manifests.expect_delete_cargo().never();
    manifests
        .expect_insert_telemetry()
        .times(1)
        .returning(|_, _, _| Ok(()));

    let mut manifest_rules = MockManifestRules::new();
    manifest_rules
        .expect_load()
        .times(1)
        .returning(|_, _, _, _| {
            Ok(StubManifestChange::upserting(
                "9Z8y",
                "aZl8",
                StubCargo::boxed("9Z8y", "aZl8", 10.0),
            ))
        });

    let mut views = MockViewCache::new();
    views
        .expect_invalidate()
        .withf(|group| group == "container")
        .times(1)
        .returning(|_| Ok(()));

    service(containers, products, manifests, manifest_rules, views)
        .load(command(user_with(&["manifest:load"])))
        .await
        .expect("o embarque não falha");
}

/// Quando o domínio manda limpar, é `clear_manifest` — e nunca `delete_cargo`.
///
/// Os três ramos são exclusivos, e confundi-los apagaria a linha de um produto
/// quando o contêiner inteiro deveria ter sido esvaziado.
#[tokio::test]
async fn mudanca_que_zera_o_conteiner_limpa_o_manifesto() {
    let mut containers = MockContainers::new();
    let mut products = MockProducts::new();
    found(&mut containers, &mut products);
    containers.expect_update().times(1).returning(|_| Ok(()));

    let mut manifests = MockManifests::new();
    manifests
        .expect_find_cargo()
        .times(1)
        .returning(|c, p| Ok(Some(StubCargo::boxed(c, p, 10.0))));
    manifests
        .expect_clear_manifest()
        .times(1)
        .returning(|_| Ok(()));
    manifests.expect_delete_cargo().never();
    manifests.expect_upsert_cargo().never();
    manifests
        .expect_insert_telemetry()
        .times(1)
        .returning(|_, _, _| Ok(()));

    let mut manifest_rules = MockManifestRules::new();
    manifest_rules
        .expect_unload()
        .times(1)
        .returning(|_, _, _, _| Ok(StubManifestChange::clearing("9Z8y", "aZl8")));

    let mut views = MockViewCache::new();
    views.expect_invalidate().times(1).returning(|_| Ok(()));

    service(containers, products, manifests, manifest_rules, views)
        .unload(command(user_with(&["manifest:unload"])))
        .await
        .expect("o desembarque não falha");
}

/// Quando o domínio manda apagar só a linha, é `delete_cargo`.
#[tokio::test]
async fn mudanca_sem_carga_apaga_so_a_linha() {
    let mut containers = MockContainers::new();
    let mut products = MockProducts::new();
    found(&mut containers, &mut products);
    containers.expect_update().times(1).returning(|_| Ok(()));

    let mut manifests = MockManifests::new();
    manifests
        .expect_find_cargo()
        .times(1)
        .returning(|c, p| Ok(Some(StubCargo::boxed(c, p, 10.0))));
    manifests
        .expect_delete_cargo()
        .withf(|container_id, product_id| container_id == "9Z8y" && product_id == "aZl8")
        .times(1)
        .returning(|_, _| Ok(()));
    manifests.expect_clear_manifest().never();
    manifests.expect_upsert_cargo().never();
    manifests
        .expect_insert_telemetry()
        .times(1)
        .returning(|_, _, _| Ok(()));

    let mut manifest_rules = MockManifestRules::new();
    manifest_rules
        .expect_unload()
        .times(1)
        .returning(|_, _, _, _| Ok(StubManifestChange::removing("9Z8y", "aZl8")));

    let mut views = MockViewCache::new();
    views.expect_invalidate().times(1).returning(|_| Ok(()));

    service(containers, products, manifests, manifest_rules, views)
        .unload(command(user_with(&["manifest:unload"])))
        .await
        .expect("o desembarque não falha");
}
