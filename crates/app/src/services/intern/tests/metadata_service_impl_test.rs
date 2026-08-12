//! Os testes de `metadata_service_impl`.

use portmaster_domain::error::{FieldError, MetadataError as DomainMetadataError};

use super::*;
use crate::tests::factories::permission_factory::StubPermission;
use crate::tests::factories::user_context_factory::user_with;
use crate::tests::mocks::permission_repository_mock::MockPermissions;
use crate::tests::mocks::permission_tm_mock::MockPermissionRules;

/// O service com os dois mocks que o teste armou.
fn service(
    permissions: MockPermissions,
    permission_rules: MockPermissionRules,
) -> MetadataServiceImpl<MockPermissions, MockPermissionRules> {
    MetadataServiceImpl::new(permissions, permission_rules)
}

/// A consulta ao catálogo, com o contexto e o filtro que o teste escolheu.
fn query(context: crate::context::UserContext, search: Option<&str>) -> ListPermissionsQuery {
    ListPermissionsQuery {
        context,
        search: search.map(str::to_owned),
    }
}

/// Registrar não confere permissão: quem chama é o boot, e não há chamador.
///
/// É deliberado, e o teste existe para que deixar de ser deliberado apareça.
#[tokio::test]
async fn registrar_nao_exige_permissao() {
    let mut permission_rules = MockPermissionRules::new();
    permission_rules
        .expect_create()
        .times(1)
        .returning(|_| Ok(StubPermission::boxed("role:create")));

    let mut permissions = MockPermissions::new();
    permissions.expect_register().times(1).returning(|_| Ok(()));

    service(permissions, permission_rules)
        .register_permission(RegisterPermissionCommand {
            slug: "role:create".to_owned(),
        })
        .await
        .expect("o boot registra sem contexto nenhum");
}

/// O que o table module recusa não chega ao catálogo.
#[tokio::test]
async fn slug_invalido_nao_chega_ao_catalogo() {
    let mut permission_rules = MockPermissionRules::new();
    permission_rules.expect_create().times(1).returning(|_| {
        Err(DomainMetadataError::Validation(vec![FieldError::new(
            "slug",
            "fora do formato recurso:ação",
        )]))
    });

    let mut permissions = MockPermissions::new();
    permissions.expect_register().never();

    service(permissions, permission_rules)
        .register_permission(RegisterPermissionCommand {
            slug: "sem-dois-pontos".to_owned(),
        })
        .await
        .expect_err("um slug fora do formato não entra no catálogo");
}

/// Sem a permissão, o catálogo não é sequer lido.
#[tokio::test]
async fn listar_sem_permissao_nao_le_o_catalogo() {
    let mut permissions = MockPermissions::new();
    permissions.expect_all().never();

    let error = service(permissions, MockPermissionRules::new())
        .list_permissions(query(user_with(&[]), None))
        .await
        .expect_err("sem a permissão, listar tem de recusar");

    assert!(matches!(
        error,
        MetadataError::App(AppError::PermissionDenied {
            permission: "permission:list"
        })
    ));
}

/// Sem filtro, sai o catálogo inteiro.
#[tokio::test]
async fn listar_sem_filtro_devolve_tudo() {
    let mut permissions = MockPermissions::new();
    permissions
        .expect_all()
        .times(1)
        .returning(|| Ok(vec!["role:create".to_owned(), "user:read".to_owned()]));

    let slugs = service(permissions, MockPermissionRules::new())
        .list_permissions(query(user_with(&["permission:list"]), None))
        .await
        .expect("listar não falha");

    assert_eq!(slugs, ["role:create", "user:read"]);
}

/// O filtro é por conteúdo e não diferencia caixa.
///
/// Um filtro só de espaços é o mesmo que filtro nenhum: quem digitou espaço não
/// pediu para não ver nada.
#[tokio::test]
async fn o_filtro_ignora_caixa_e_espaco_em_branco() {
    let catalog = || Ok(vec!["role:create".to_owned(), "user:read".to_owned()]);

    let mut permissions = MockPermissions::new();
    permissions.expect_all().times(1).returning(catalog);

    let filtered = service(permissions, MockPermissionRules::new())
        .list_permissions(query(user_with(&["permission:list"]), Some("ROLE")))
        .await
        .expect("listar não falha");

    assert_eq!(filtered, ["role:create"]);

    let mut permissions = MockPermissions::new();
    permissions.expect_all().times(1).returning(catalog);

    let blank = service(permissions, MockPermissionRules::new())
        .list_permissions(query(user_with(&["permission:list"]), Some("   ")))
        .await
        .expect("listar não falha");

    assert_eq!(blank, ["role:create", "user:read"]);
}
