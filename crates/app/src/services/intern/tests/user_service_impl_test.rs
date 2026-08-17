//! Os testes de `user_service_impl`.

use portmaster_domain::error::{FieldError, UserError as DomainUserError};

use super::*;
use crate::event::EventProvider;
use crate::tests::factories::role_factory::StubRole;
use crate::tests::factories::user_context_factory::user_with;
use crate::tests::factories::user_factory::StubUser;
use crate::tests::mocks::query_repository_stub::StubQueries;
use crate::tests::mocks::role_repository_mock::MockRoles;
use crate::tests::mocks::user_repository_mock::MockUsers;
use crate::tests::mocks::user_tm_mock::MockUserRules;
use crate::tests::mocks::view_cache_repository_mock::MockViewCache;

/// O service com os mocks que o teste armou.
fn service(
    users: MockUsers,
    roles: MockRoles,
    user_rules: MockUserRules,
    views: MockViewCache,
) -> impl UserService {
    user_service(
        users,
        roles,
        user_rules,
        StubQueries::never(),
        views,
        EventProvider::meta_event_stack(),
    )
}

/// O comando de criação, com o contexto e os papéis que o teste escolheu.
fn create_command(context: crate::context::UserContext, role_ids: &[&str]) -> CreateUserCommand {
    CreateUserCommand {
        context,
        name: "Fulano".to_owned(),
        email: "fulano@exemplo.com".to_owned(),
        initial_password: "senha-longa-o-bastante".to_owned(),
        role_ids: role_ids.iter().map(|id| (*id).to_owned()).collect(),
    }
}

/// Sem a permissão, a recusa acontece antes de qualquer port ser tocada.
#[tokio::test]
async fn criar_sem_permissao_nao_toca_em_port_nenhuma() {
    let mut users = MockUsers::new();
    users.expect_find_by_email().never();

    let Err(error) = service(
        users,
        MockRoles::new(),
        MockUserRules::new(),
        MockViewCache::new(),
    )
    .create(create_command(user_with(&[]), &[]))
    .await
    else {
        panic!("sem a permissão, criar tem de recusar");
    };

    assert!(matches!(
        error,
        UserError::App(AppError::PermissionDenied {
            permission: "user:create"
        })
    ));
}

/// O caminho feliz grava o usuário **e** a tabela de ligação dos papéis.
///
/// São duas escritas na mesma transação, e o `sync_roles` é a que se esquece: um
/// usuário criado sem ela nasce sem papel nenhum e sem erro nenhum.
#[tokio::test]
async fn criar_grava_o_usuario_e_os_papeis() {
    let mut roles = MockRoles::new();
    roles
        .expect_find_by_id()
        .times(1)
        .returning(|id| Ok(Some(StubRole::boxed(id, &[]))));

    let mut user_rules = MockUserRules::new();
    user_rules
        .expect_create()
        .times(1)
        .returning(|_, _, _, _| Ok(StubUser::boxed("9Z8y", "fulano@exemplo.com")));

    let mut users = MockUsers::new();
    users
        .expect_find_by_email()
        .times(1)
        .returning(|_| Ok(None));
    users.expect_insert().times(1).returning(|_| Ok(()));
    users
        .expect_sync_roles()
        .withf(|user_id, role_ids| user_id == "9Z8y" && role_ids == ["bYk7X1"])
        .times(1)
        .returning(|_, _| Ok(()));

    let mut views = MockViewCache::new();
    views
        .expect_invalidate()
        .withf(|group| group == "user")
        .times(1)
        .returning(|_| Ok(()));

    let Ok(user) = service(users, roles, user_rules, views)
        .create(create_command(user_with(&["user:create"]), &["bYk7X1"]))
        .await
    else {
        panic!("o caminho feliz não falha");
    };

    assert_eq!(user.id(), "9Z8y");
}

/// E-mail já usado é recusado **antes** de o table module construir.
///
/// Descoberto aqui em vez de deixar o índice único reclamar: o cliente recebe o
/// campo que precisa corrigir em vez de um erro de banco.
#[tokio::test]
async fn email_ja_usado_nao_chega_ao_table_module() {
    let mut users = MockUsers::new();
    users
        .expect_find_by_email()
        .times(1)
        .returning(|email| Ok(Some(StubUser::boxed("outro", email))));
    users.expect_insert().never();
    users.expect_sync_roles().never();

    let mut user_rules = MockUserRules::new();
    user_rules.expect_create().never();

    let mut views = MockViewCache::new();
    views.expect_invalidate().never();

    let Err(error) = service(users, MockRoles::new(), user_rules, views)
        .create(create_command(user_with(&["user:create"]), &[]))
        .await
    else {
        panic!("e-mail repetido tem de recusar");
    };

    assert!(matches!(error, UserError::EmailTaken));
}

/// Um papel inexistente recusa a criação inteira, e nada é gravado.
#[tokio::test]
async fn papel_inexistente_recusa_a_criacao() {
    let mut users = MockUsers::new();
    users
        .expect_find_by_email()
        .times(1)
        .returning(|_| Ok(None));
    users.expect_insert().never();

    let mut roles = MockRoles::new();
    roles.expect_find_by_id().times(1).returning(|_| Ok(None));

    let mut views = MockViewCache::new();
    views.expect_invalidate().never();

    let Err(error) = service(users, roles, MockUserRules::new(), views)
        .create(create_command(user_with(&["user:create"]), &["nao-existe"]))
        .await
    else {
        panic!("papel inexistente recusa");
    };

    assert!(matches!(error, UserError::MissingRole(id) if id == "nao-existe"));
}

/// O que o table module recusa não chega ao repositório.
#[tokio::test]
async fn regra_recusada_nao_chega_ao_repositorio() {
    let mut users = MockUsers::new();
    users
        .expect_find_by_email()
        .times(1)
        .returning(|_| Ok(None));
    users.expect_insert().never();
    users.expect_sync_roles().never();

    let mut user_rules = MockUserRules::new();
    user_rules.expect_create().times(1).returning(|_, _, _, _| {
        Err(DomainUserError::Validation(vec![FieldError::new(
            "password",
            "senha curta demais",
        )]))
    });

    let mut views = MockViewCache::new();
    views.expect_invalidate().never();

    let Err(error) = service(users, MockRoles::new(), user_rules, views)
        .create(create_command(user_with(&["user:create"]), &[]))
        .await
    else {
        panic!("a regra recusou");
    };

    assert!(matches!(error, UserError::App(AppError::Validation(_))));
}
