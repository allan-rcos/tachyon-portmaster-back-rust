//! Os testes de `container_service_impl`.
//!
//! `seal` e `dispatch` compartilham o `transition`, e é ele que este arquivo
//! exercita: quem recusa a transição é o `ContainerTM`, não o service — o
//! service nem conhece o status exigido. O que se afirma aqui é que a recusa do
//! table module impede a gravação, e não que a regra em si esteja certa.

use portmaster_domain::enums::ContainerStatus;
use portmaster_domain::error::ContainerError as DomainContainerError;

use super::*;
use crate::tests::factories::container_factory::StubContainer;
use crate::tests::factories::user_context_factory::user_with;
use crate::tests::mocks::container_repository_mock::MockContainers;
use crate::tests::mocks::container_tm_mock::MockContainerRules;
use crate::tests::mocks::query_repository_stub::StubQueries;
use crate::tests::mocks::view_cache_repository_mock::MockViewCache;

/// O service com os mocks que o teste armou.
fn service(
    containers: MockContainers,
    container_rules: MockContainerRules,
    views: MockViewCache,
) -> ContainerServiceImpl<MockContainers, MockContainerRules, StubQueries, MockViewCache> {
    ContainerServiceImpl::new(containers, container_rules, StubQueries::never(), views)
}

/// O comando que carrega só id e contexto.
fn command(context: crate::context::UserContext) -> ContainerCommand {
    ContainerCommand {
        context,
        id: "9Z8y".to_owned(),
    }
}

/// Sem a permissão, a recusa acontece antes de qualquer port ser tocada.
#[tokio::test]
async fn selar_sem_permissao_nao_toca_em_port_nenhuma() {
    let mut containers = MockContainers::new();
    containers.expect_find_by_id().never();

    let error = service(containers, MockContainerRules::new(), MockViewCache::new())
        .seal(command(user_with(&[])))
        .await
        .expect_err("sem a permissão, selar tem de recusar");

    assert!(matches!(
        error,
        ContainerError::App(AppError::PermissionDenied {
            permission: "container:seal"
        })
    ));
}

/// O caminho feliz: o table module move o status, o repositório grava, o cache cai.
#[tokio::test]
async fn selar_grava_e_derruba_o_cache() {
    let mut containers = MockContainers::new();
    containers
        .expect_find_by_id()
        .times(1)
        .returning(|_| Ok(Some(StubContainer::boxed("9Z8y", ContainerStatus::Loading))));
    containers.expect_update().times(1).returning(|_| Ok(()));

    let mut container_rules = MockContainerRules::new();
    container_rules
        .expect_seal()
        .times(1)
        .returning(|_| Ok(StubContainer::boxed("9Z8y", ContainerStatus::Sealed)));

    let mut views = MockViewCache::new();
    views
        .expect_invalidate()
        .withf(|group| group == "container")
        .times(1)
        .returning(|_| Ok(()));

    service(containers, container_rules, views)
        .seal(command(user_with(&["container:seal"])))
        .await
        .expect("o caminho feliz não falha");
}

/// A transição recusada pelo table module não chega ao repositório.
///
/// Despachar um contêiner que não está selado é o caso: quem sabe disso é o
/// `ContainerTM`, e o service só precisa não gravar o que ele recusou.
#[tokio::test]
async fn transicao_recusada_nao_grava_nem_derruba_o_cache() {
    let mut containers = MockContainers::new();
    containers
        .expect_find_by_id()
        .times(1)
        .returning(|_| Ok(Some(StubContainer::boxed("9Z8y", ContainerStatus::Loading))));
    containers.expect_update().never();

    let mut container_rules = MockContainerRules::new();
    container_rules
        .expect_dispatch()
        .times(1)
        .returning(|_| Err(DomainContainerError::DispatchRequiresSealed));

    let mut views = MockViewCache::new();
    views.expect_invalidate().never();

    service(containers, container_rules, views)
        .dispatch(command(user_with(&["container:dispatch"])))
        .await
        .expect_err("despachar sem selar tem de recusar");
}

/// Transicionar um contêiner que não existe é `Missing`, e nada é gravado.
#[tokio::test]
async fn transicao_de_conteiner_inexistente_nao_grava() {
    let mut containers = MockContainers::new();
    containers
        .expect_find_by_id()
        .times(1)
        .returning(|_| Ok(None));
    containers.expect_update().never();

    let mut container_rules = MockContainerRules::new();
    container_rules.expect_seal().never();

    let mut views = MockViewCache::new();
    views.expect_invalidate().never();

    let error = service(containers, container_rules, views)
        .seal(command(user_with(&["container:seal"])))
        .await
        .expect_err("contêiner inexistente recusa");

    assert!(matches!(error, ContainerError::Missing(id) if id == "9Z8y"));
}
