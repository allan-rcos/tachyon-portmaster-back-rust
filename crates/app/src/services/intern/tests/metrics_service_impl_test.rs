//! Os testes de `metrics_service_impl`.
//!
//! É o service mais simples que tem cache de leitura, e por isso é aqui que as
//! duas metades desse caminho ficam fixadas: acerto no cache **não** consulta o
//! banco, e erro no cache consulta e guarda.

use portmaster_infra::query::views::MetricsView;

use super::*;
use crate::tests::factories::user_context_factory::user_with;
use crate::tests::mocks::query_repository_stub::StubQueries;
use crate::tests::mocks::view_cache_repository_mock::MockViewCache;

/// O service com os dois mocks que o teste armou.
fn service(
    queries: StubQueries,
    views: MockViewCache,
) -> MetricsServiceImpl<StubQueries, MockViewCache> {
    MetricsServiceImpl::new(queries, views)
}

/// Um painel reconhecível, para a asserção não depender de zeros.
fn view() -> MetricsView {
    MetricsView {
        total_containers: 7,
        ..MetricsView::default()
    }
}

/// Sem a permissão, nem o cache nem o banco são tocados.
#[tokio::test]
async fn ler_sem_permissao_nao_toca_em_port_nenhuma() {
    let mut views = MockViewCache::new();
    views.expect_get::<MetricsView>().never();

    let error = service(StubQueries::never(), views)
        .get(GetMetricsQuery {
            context: user_with(&[]),
        })
        .await
        .expect_err("sem a permissão, ler tem de recusar");

    assert!(matches!(
        error,
        MetricsError::App(AppError::PermissionDenied {
            permission: "metrics:read"
        })
    ));
}

/// Acerto no cache responde sem consultar o banco.
///
/// É a razão de o cache existir: oito agregações varrendo as tabelas inteiras,
/// pedidas a cada carregamento de tela.
#[tokio::test]
async fn acerto_no_cache_nao_consulta_o_banco() {
    let mut views = MockViewCache::new();
    views
        .expect_get::<MetricsView>()
        .times(1)
        .returning(|_, _| Ok(Some(view())));
    views.expect_put::<MetricsView>().never();

    let panel = service(StubQueries::never(), views)
        .get(GetMetricsQuery {
            context: user_with(&["metrics:read"]),
        })
        .await
        .expect("o acerto no cache responde");

    assert_eq!(panel.total_containers, 7);
}

/// Erro no cache consulta o banco e guarda o que veio.
#[tokio::test]
async fn erro_no_cache_consulta_e_guarda() {
    let mut views = MockViewCache::new();
    views
        .expect_get::<MetricsView>()
        .times(1)
        .returning(|_, _| Ok(None));
    views
        .expect_put::<MetricsView>()
        .withf(|group, _, panel| group == "metrics" && panel.total_containers == 7)
        .times(1)
        .returning(|_, _, _| Ok(()));

    let queries = StubQueries::returning(view());

    let panel = service(queries, views)
        .get(GetMetricsQuery {
            context: user_with(&["metrics:read"]),
        })
        .await
        .expect("a consulta responde");

    assert_eq!(panel.total_containers, 7);
}
