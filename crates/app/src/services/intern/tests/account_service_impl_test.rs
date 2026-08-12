//! Os testes de `account_service_impl`.
//!
//! Este service age sobre **quem pediu**, e não sobre um id do corpo: por isso
//! nenhum método confere permissão, e todos partem de `command.context.id`. O
//! que se afirma aqui é isso, e a exigência da senha atual para trocá-la.

use portmaster_domain::error::AuthError;

use super::*;
use crate::tests::factories::user_context_factory::user_with;
use crate::tests::factories::user_factory::StubUser;
use crate::tests::mocks::auth_tm_mock::MockAuthRules;
use crate::tests::mocks::query_repository_stub::StubQueries;
use crate::tests::mocks::user_repository_mock::MockUsers;
use crate::tests::mocks::user_tm_mock::MockUserRules;
use crate::tests::mocks::view_cache_repository_mock::MockViewCache;

/// O service com os mocks que o teste armou.
fn service(
    users: MockUsers,
    user_rules: MockUserRules,
    auth_rules: MockAuthRules,
    views: MockViewCache,
) -> AccountServiceImpl<MockUsers, MockUserRules, MockAuthRules, StubQueries, MockViewCache> {
    AccountServiceImpl::new(users, user_rules, auth_rules, StubQueries::never(), views)
}

/// O comando de troca de senha, com a senha atual que o teste escolheu.
fn change_command(current: &str) -> ChangePasswordCommand {
    ChangePasswordCommand {
        context: user_with(&[]),
        current_password: current.to_owned(),
        new_password: "a-senha-nova-longa".to_owned(),
    }
}

/// A conta é procurada pelo id **do contexto**, e não por um id do corpo.
///
/// É o que impede o endpoint de virar uma edição de conta alheia: o corpo não
/// tem onde dizer sobre quem se age.
#[tokio::test]
async fn a_conta_editada_e_a_de_quem_pediu() {
    let context = user_with(&[]);
    let expected = context.id.clone();

    let mut users = MockUsers::new();
    users
        .expect_find_by_id()
        .withf(move |id| id == expected)
        .times(1)
        .returning(|id| Ok(Some(StubUser::boxed(id, "fulano@exemplo.com"))));
    users.expect_update().times(1).returning(|_| Ok(()));

    let mut user_rules = MockUserRules::new();
    user_rules
        .expect_update()
        .times(1)
        .returning(|_, _, email| Ok(StubUser::boxed("aZl8Y0", &email)));

    let mut views = MockViewCache::new();
    views
        .expect_invalidate()
        .withf(|group| group == "account")
        .times(1)
        .returning(|_| Ok(()));

    let Ok(user) = service(users, user_rules, MockAuthRules::new(), views)
        .update(UpdateAccountCommand {
            context,
            name: "Fulano".to_owned(),
            email: "novo@exemplo.com".to_owned(),
        })
        .await
    else {
        panic!("editar a própria conta não falha");
    };

    assert_eq!(user.email(), "novo@exemplo.com");
}

/// A senha atual é exigida mesmo com a sessão válida.
///
/// Um token roubado não deve bastar para trocar a senha e expulsar o dono — e é
/// por isso que a recusa acontece **antes** de o table module derivar o hash
/// novo.
#[tokio::test]
async fn senha_atual_errada_nao_troca_a_senha() {
    let mut users = MockUsers::new();
    users
        .expect_find_by_id()
        .times(1)
        .returning(|id| Ok(Some(StubUser::boxed(id, "fulano@exemplo.com"))));
    users.expect_update().never();

    let mut auth_rules = MockAuthRules::new();
    auth_rules
        .expect_login()
        .times(1)
        .returning(|_, _| Err(AuthError::InvalidCredentials));

    let mut user_rules = MockUserRules::new();
    user_rules.expect_change_password().never();

    let mut views = MockViewCache::new();
    views.expect_invalidate().never();

    let error = service(users, user_rules, auth_rules, views)
        .change_password(change_command("a-senha-errada"))
        .await
        .expect_err("senha atual errada recusa");

    assert!(matches!(error, AccountError::InvalidCredentials));
}

/// Com a senha atual certa, a nova é gravada e o cache cai.
#[tokio::test]
async fn senha_atual_certa_troca_a_senha() {
    let mut users = MockUsers::new();
    users
        .expect_find_by_id()
        .times(1)
        .returning(|id| Ok(Some(StubUser::boxed(id, "fulano@exemplo.com"))));
    users.expect_update().times(1).returning(|_| Ok(()));

    let mut auth_rules = MockAuthRules::new();
    auth_rules.expect_login().times(1).returning(|_, _| Ok(()));

    let mut user_rules = MockUserRules::new();
    user_rules
        .expect_change_password()
        .times(1)
        .returning(|_, _| Ok(StubUser::boxed("aZl8Y0", "fulano@exemplo.com")));

    let mut views = MockViewCache::new();
    views.expect_invalidate().times(1).returning(|_| Ok(()));

    service(users, user_rules, auth_rules, views)
        .change_password(change_command("a-senha-certa"))
        .await
        .expect("a troca de senha não falha");
}

/// Conta que sumiu entre a emissão do token e o pedido é credencial inválida.
///
/// O token continua assinado e válido, e não há mais dono para ele.
#[tokio::test]
async fn conta_removida_e_credencial_invalida() {
    let mut users = MockUsers::new();
    users.expect_find_by_id().times(1).returning(|_| Ok(None));
    users.expect_update().never();

    let mut views = MockViewCache::new();
    views.expect_invalidate().never();

    let error = service(users, MockUserRules::new(), MockAuthRules::new(), views)
        .change_password(change_command("qualquer"))
        .await
        .expect_err("sem dono, a sessão não vale");

    assert!(matches!(error, AccountError::InvalidCredentials));
}
