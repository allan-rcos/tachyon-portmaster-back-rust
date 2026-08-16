//! Os testes de `session_service_impl`.
//!
//! Este arquivo guarda duas regras de segurança, e é por elas que ele existe:
//! e-mail desconhecido e senha errada produzem o **mesmo** erro, e o `setup` só
//! monta o primeiro usuário do sistema.

use portmaster_domain::error::AuthError;

use super::*;
use crate::tests::factories::role_factory::StubRole;
use crate::tests::factories::user_factory::StubUser;
use crate::tests::mocks::auth_tm_mock::MockAuthRules;
use crate::tests::mocks::permission_repository_mock::MockPermissions;
use crate::tests::mocks::role_repository_mock::MockRoles;
use crate::tests::mocks::role_tm_mock::MockRoleRules;
use crate::tests::mocks::user_repository_mock::MockUsers;
use crate::tests::mocks::user_tm_mock::MockUserRules;

/// O service com os mocks que o teste armou.
fn service(
    users: MockUsers,
    roles: MockRoles,
    permissions: MockPermissions,
    user_rules: MockUserRules,
    role_rules: MockRoleRules,
    auth_rules: MockAuthRules,
) -> impl SessionService {
    session_service(
        users,
        roles,
        permissions,
        user_rules,
        role_rules,
        auth_rules,
    )
}

/// O service com tudo em branco, para os testes que só usam uma port.
fn with_users(users: MockUsers, auth_rules: MockAuthRules) -> impl SessionService {
    service(
        users,
        MockRoles::new(),
        MockPermissions::new(),
        MockUserRules::new(),
        MockRoleRules::new(),
        auth_rules,
    )
}

/// O comando de login.
fn login_command() -> LoginCommand {
    LoginCommand {
        email: "fulano@exemplo.com".to_owned(),
        password: "a-senha".to_owned(),
    }
}

/// E-mail desconhecido e senha errada dão **o mesmo** erro.
///
/// É a regra que impede a tela de login de virar um verificador de quem tem
/// conta no sistema. Os dois caminhos são exercitados no mesmo teste de
/// propósito: a asserção é que eles não se distinguem.
#[tokio::test]
async fn email_desconhecido_e_senha_errada_sao_indistinguiveis() {
    let mut unknown = MockUsers::new();
    unknown
        .expect_find_by_email()
        .times(1)
        .returning(|_| Ok(None));

    let Err(by_email) = with_users(unknown, MockAuthRules::new())
        .login(login_command())
        .await
    else {
        panic!("e-mail desconhecido recusa");
    };

    let mut known = MockUsers::new();
    known
        .expect_find_by_email()
        .times(1)
        .returning(|email| Ok(Some(StubUser::boxed("9Z8y", email))));

    let mut auth_rules = MockAuthRules::new();
    auth_rules
        .expect_login()
        .times(1)
        .returning(|_, _| Err(AuthError::InvalidCredentials));

    let Err(by_password) = with_users(known, auth_rules).login(login_command()).await else {
        panic!("senha errada recusa");
    };

    assert!(matches!(by_email, SessionError::InvalidCredentials));
    assert!(matches!(by_password, SessionError::InvalidCredentials));
    assert_eq!(by_email.to_string(), by_password.to_string());
}

/// O login que passa devolve o usuário.
#[tokio::test]
async fn credencial_valida_devolve_o_usuario() {
    let mut users = MockUsers::new();
    users
        .expect_find_by_email()
        .times(1)
        .returning(|email| Ok(Some(StubUser::boxed("9Z8y", email))));

    let mut auth_rules = MockAuthRules::new();
    auth_rules.expect_login().times(1).returning(|_, _| Ok(()));

    let Ok(user) = with_users(users, auth_rules).login(login_command()).await else {
        panic!("a credencial válida entra");
    };

    assert_eq!(user.id(), "9Z8y");
}

/// Com **qualquer** usuário no sistema, o `setup` recusa e nada é gravado.
///
/// Sem essa checagem o endpoint seria uma porta aberta para criar um
/// administrador a qualquer momento.
#[tokio::test]
async fn setup_num_sistema_ja_montado_nao_grava() {
    let mut users = MockUsers::new();
    users.expect_has_any().times(1).returning(|| Ok(true));
    users.expect_insert().never();
    users.expect_sync_roles().never();

    let mut roles = MockRoles::new();
    roles.expect_insert().never();

    let mut role_rules = MockRoleRules::new();
    role_rules.expect_create().never();

    let Err(error) = service(
        users,
        roles,
        MockPermissions::new(),
        MockUserRules::new(),
        role_rules,
        MockAuthRules::new(),
    )
    .setup(SetupCommand {
        name: "Admin".to_owned(),
        email: "admin@exemplo.com".to_owned(),
        password: "senha-longa-o-bastante".to_owned(),
    })
    .await
    else {
        panic!("um sistema já montado recusa o setup");
    };

    assert!(matches!(error, SessionError::AlreadySetUp));
}

/// O administrador nasce com o catálogo inteiro, e não com uma lista literal.
///
/// É o que faz uma permissão nova ser concedida ao administrador sem ninguém
/// lembrar de voltar ao service.
#[tokio::test]
async fn o_setup_concede_o_catalogo_registrado() {
    let mut users = MockUsers::new();
    users.expect_has_any().times(1).returning(|| Ok(false));
    users.expect_insert().times(1).returning(|_| Ok(()));
    users
        .expect_sync_roles()
        .withf(|_, role_ids| role_ids == ["bYk7X1"])
        .times(1)
        .returning(|_, _| Ok(()));

    let mut permissions = MockPermissions::new();
    permissions
        .expect_all()
        .times(1)
        .returning(|| Ok(vec!["role:create".to_owned(), "user:read".to_owned()]));

    let mut role_rules = MockRoleRules::new();
    role_rules
        .expect_create()
        .withf(|name, granted| name == "Administrator" && *granted == ["role:create", "user:read"])
        .times(1)
        .returning(|_, granted| {
            let slugs: Vec<&str> = granted.iter().map(String::as_str).collect();
            Ok(StubRole::boxed("bYk7X1", &slugs))
        });

    let mut roles = MockRoles::new();
    roles.expect_insert().times(1).returning(|_| Ok(()));

    let mut user_rules = MockUserRules::new();
    user_rules
        .expect_create()
        .times(1)
        .returning(|_, email, _, _| Ok(StubUser::boxed("9Z8y", &email)));

    let Ok(user) = service(
        users,
        roles,
        permissions,
        user_rules,
        role_rules,
        MockAuthRules::new(),
    )
    .setup(SetupCommand {
        name: "Admin".to_owned(),
        email: "admin@exemplo.com".to_owned(),
        password: "senha-longa-o-bastante".to_owned(),
    })
    .await
    else {
        panic!("o primeiro setup monta o sistema");
    };

    assert_eq!(user.id(), "9Z8y");
}
