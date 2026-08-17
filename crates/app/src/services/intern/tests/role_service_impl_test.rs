//! Os testes de `role_service_impl`.
//!
//! O que se afirma aqui é **orquestração**: quem é chamado, em que ordem, e o
//! que deixa de ser chamado quando algo recusa. A regra de papel em si é do
//! `RoleTM` e já tem teste no `domain`; aqui ele é mock justamente para que uma
//! falha de regra não se confunda com uma falha de orquestração.
//!
//! Um mock sem expectativa entra em pânico se for chamado. É isso que faz
//! "nenhum mock foi tocado" ser uma asserção de verdade, e não uma esperança.

use portmaster_domain::error::{FieldError, RoleError as DomainRoleError};

use super::*;
use crate::event::EventProvider;
use crate::tests::factories::role_factory::StubRole;
use crate::tests::factories::user_context_factory::user_with;
use crate::tests::mocks::query_repository_stub::StubQueries;
use crate::tests::mocks::role_repository_mock::MockRoles;
use crate::tests::mocks::role_tm_mock::MockRoleRules;
use crate::tests::mocks::view_cache_repository_mock::MockViewCache;

/// O service com os quatro mocks que o teste armou.
fn service(roles: MockRoles, role_rules: MockRoleRules, views: MockViewCache) -> impl RoleService {
    role_service(
        roles,
        role_rules,
        StubQueries::never(),
        views,
        EventProvider::meta_event_stack(),
    )
}

/// O comando de criação, com o contexto que o teste escolheu.
fn create_command(context: crate::context::UserContext) -> CreateRoleCommand {
    CreateRoleCommand {
        context,
        name: "Operador".to_owned(),
        permissions: vec!["container:list".to_owned()],
    }
}

/// Sem a permissão, a recusa acontece **antes** de qualquer port ser tocada.
///
/// É a asserção que protege a ordem: conferir a permissão depois de gravar
/// deixaria o efeito colateral acontecer e só então recusar.
#[tokio::test]
async fn criar_sem_permissao_nao_toca_em_port_nenhuma() {
    let service = service(MockRoles::new(), MockRoleRules::new(), MockViewCache::new());

    let Err(error) = service.create(create_command(user_with(&[]))).await else {
        panic!("sem a permissão, criar tem de recusar");
    };

    assert!(matches!(
        error,
        RoleError::App(AppError::PermissionDenied {
            permission: "role:create"
        })
    ));
}

/// O caminho feliz: o table module constrói, o repositório grava, o cache cai.
#[tokio::test]
async fn criar_grava_e_derruba_o_cache() {
    let mut role_rules = MockRoleRules::new();
    role_rules
        .expect_create()
        .times(1)
        .returning(|_, _| Ok(StubRole::boxed("9Z8y", &["container:list"])));

    let mut roles = MockRoles::new();
    roles.expect_insert().times(1).returning(|_| Ok(()));

    let mut views = MockViewCache::new();
    views
        .expect_invalidate()
        .withf(|group| group == "role")
        .times(1)
        .returning(|_| Ok(()));

    let role = service(roles, role_rules, views)
        .create(create_command(user_with(&["role:create"])))
        .await
        .expect("o caminho feliz não falha");

    assert_eq!(role.id(), "9Z8y");
}

/// Se a gravação falha, o cache **não** é derrubado.
///
/// Derrubá-lo assim mesmo descartaria leitura boa por causa de uma escrita que
/// não aconteceu — e o próximo pedido recalcularia tudo para chegar ao mesmo
/// resultado anterior.
#[tokio::test]
async fn falha_ao_gravar_nao_derruba_o_cache() {
    let mut role_rules = MockRoleRules::new();
    role_rules
        .expect_create()
        .returning(|_, _| Ok(StubRole::boxed("9Z8y", &[])));

    let mut roles = MockRoles::new();
    roles
        .expect_insert()
        .times(1)
        .returning(|_| Err(anyhow::anyhow!("o banco recusou")));

    let mut views = MockViewCache::new();
    views.expect_invalidate().never();

    let Err(error) = service(roles, role_rules, views)
        .create(create_command(user_with(&["role:create"])))
        .await
    else {
        panic!("a falha do repositório tem de subir");
    };

    assert!(matches!(error, RoleError::App(AppError::Infra(_))));
}

/// O que o table module recusa não chega ao repositório.
#[tokio::test]
async fn regra_recusada_nao_chega_ao_repositorio() {
    let mut role_rules = MockRoleRules::new();
    role_rules.expect_create().times(1).returning(|_, _| {
        Err(DomainRoleError::Validation(vec![FieldError::new(
            "name",
            "nome de papel inválido",
        )]))
    });

    let mut roles = MockRoles::new();
    roles.expect_insert().never();

    let mut views = MockViewCache::new();
    views.expect_invalidate().never();

    let Err(error) = service(roles, role_rules, views)
        .create(create_command(user_with(&["role:create"])))
        .await
    else {
        panic!("a regra recusou");
    };

    assert!(matches!(error, RoleError::App(AppError::Validation(_))));
}

/// Trocar permissões de um papel que não existe é `Missing`, e nada é gravado.
#[tokio::test]
async fn trocar_permissoes_de_papel_inexistente_nao_grava() {
    let mut roles = MockRoles::new();
    roles.expect_find_by_id().times(1).returning(|_| Ok(None));
    roles.expect_update().never();

    let mut views = MockViewCache::new();
    views.expect_invalidate().never();

    let Err(error) = service(roles, MockRoleRules::new(), views)
        .update_permissions(UpdateRolePermissionsCommand {
            context: user_with(&["role:update-permissions"]),
            id: "9Z8y".to_owned(),
            permissions: vec!["container:list".to_owned()],
        })
        .await
    else {
        panic!("papel inexistente recusa");
    };

    assert!(matches!(error, RoleError::Missing(id) if id == "9Z8y"));
}
