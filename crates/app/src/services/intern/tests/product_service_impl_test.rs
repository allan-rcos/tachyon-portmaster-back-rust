//! Os testes de `product_service_impl`.

use portmaster_domain::enums::RiskClass;
use portmaster_domain::error::{FieldError, ProductError as DomainProductError};

use super::*;
use crate::tests::factories::product_factory::StubProduct;
use crate::tests::factories::user_context_factory::user_with;
use crate::tests::mocks::product_repository_mock::MockProducts;
use crate::tests::mocks::product_tm_mock::MockProductRules;
use crate::tests::mocks::query_repository_stub::StubQueries;
use crate::tests::mocks::view_cache_repository_mock::MockViewCache;

/// O service com os mocks que o teste armou.
fn service(
    products: MockProducts,
    product_rules: MockProductRules,
    views: MockViewCache,
) -> ProductServiceImpl<MockProducts, MockProductRules, StubQueries, MockViewCache> {
    ProductServiceImpl::new(products, product_rules, StubQueries::never(), views)
}

/// O comando de cadastro, com o contexto que o teste escolheu.
fn create_command(context: crate::context::UserContext) -> CreateProductCommand {
    CreateProductCommand {
        context,
        name: "Soja".to_owned(),
        density: 0.75,
        risk_class: RiskClass::None,
    }
}

/// Sem a permissão, a recusa acontece antes de qualquer port ser tocada.
#[tokio::test]
async fn cadastrar_sem_permissao_nao_toca_em_port_nenhuma() {
    let Err(error) = service(
        MockProducts::new(),
        MockProductRules::new(),
        MockViewCache::new(),
    )
    .create(create_command(user_with(&[])))
    .await
    else {
        panic!("sem a permissão, cadastrar tem de recusar");
    };

    assert!(matches!(
        error,
        ProductError::App(AppError::PermissionDenied {
            permission: "product:create"
        })
    ));
}

/// O caminho feliz: o table module constrói, o repositório grava, o cache cai.
#[tokio::test]
async fn cadastrar_grava_e_derruba_o_cache() {
    let mut product_rules = MockProductRules::new();
    product_rules
        .expect_create()
        .times(1)
        .returning(|_, _, _| Ok(StubProduct::boxed("9Z8y", 0.75)));

    let mut products = MockProducts::new();
    products.expect_insert().times(1).returning(|_| Ok(()));

    let mut views = MockViewCache::new();
    views
        .expect_invalidate()
        .withf(|group| group == "product")
        .times(1)
        .returning(|_| Ok(()));

    let Ok(product) = service(products, product_rules, views)
        .create(create_command(user_with(&["product:create"])))
        .await
    else {
        panic!("o caminho feliz não falha");
    };

    assert_eq!(product.id(), "9Z8y");
}

/// Se a gravação falha, o cache não é derrubado.
#[tokio::test]
async fn falha_ao_gravar_nao_derruba_o_cache() {
    let mut product_rules = MockProductRules::new();
    product_rules
        .expect_create()
        .returning(|_, _, _| Ok(StubProduct::boxed("9Z8y", 0.75)));

    let mut products = MockProducts::new();
    products
        .expect_insert()
        .times(1)
        .returning(|_| Err(anyhow::anyhow!("o banco recusou")));

    let mut views = MockViewCache::new();
    views.expect_invalidate().never();

    let Err(error) = service(products, product_rules, views)
        .create(create_command(user_with(&["product:create"])))
        .await
    else {
        panic!("a falha do repositório tem de subir");
    };

    assert!(matches!(error, ProductError::App(AppError::Infra(_))));
}

/// O que o table module recusa não chega ao repositório.
#[tokio::test]
async fn regra_recusada_nao_chega_ao_repositorio() {
    let mut product_rules = MockProductRules::new();
    product_rules.expect_create().times(1).returning(|_, _, _| {
        Err(DomainProductError::Validation(vec![FieldError::new(
            "density",
            "densidade tem de ser positiva",
        )]))
    });

    let mut products = MockProducts::new();
    products.expect_insert().never();

    let mut views = MockViewCache::new();
    views.expect_invalidate().never();

    let Err(error) = service(products, product_rules, views)
        .create(create_command(user_with(&["product:create"])))
        .await
    else {
        panic!("a regra recusou");
    };

    assert!(matches!(error, ProductError::App(AppError::Validation(_))));
}

/// Remover o que não existe é `Missing`, e nada é apagado.
///
/// Sem a conferência, remover duas vezes responderia sucesso na segunda, e o
/// cliente não teria como saber que o id que ele mandou nunca existiu.
#[tokio::test]
async fn remover_o_que_nao_existe_nao_apaga() {
    let mut products = MockProducts::new();
    products
        .expect_find_by_id()
        .times(1)
        .returning(|_| Ok(None));
    products.expect_delete().never();

    let mut views = MockViewCache::new();
    views.expect_invalidate().never();

    let Err(error) = service(products, MockProductRules::new(), views)
        .delete(DeleteProductCommand {
            context: user_with(&["product:delete"]),
            id: "9Z8y".to_owned(),
        })
        .await
    else {
        panic!("produto inexistente recusa");
    };

    assert!(matches!(error, ProductError::Missing(id) if id == "9Z8y"));
}
