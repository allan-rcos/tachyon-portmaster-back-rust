//! Tudo que o boot registra e que o `POST /setup` concede.

use crate::security::permission_slug::PermissionSlug;

/// O catálogo de permissões do sistema.
///
/// Ler daqui em vez de listar à mão no setup é o que faz uma permissão nova ser
/// concedida ao administrador sem ninguém lembrar de voltar aqui.
pub struct PermissionCatalog;

impl PermissionCatalog {
    /// Todas as permissões que o boot registra.
    pub const ALL: &'static [&'static str] = &[
        PermissionSlug::CONTAINER_CREATE,
        PermissionSlug::CONTAINER_DELETE,
        PermissionSlug::CONTAINER_DISPATCH,
        PermissionSlug::CONTAINER_READ,
        PermissionSlug::CONTAINER_SEAL,
        PermissionSlug::CONTAINER_SUMMARY,
        PermissionSlug::CONTAINER_UPDATE,
        PermissionSlug::MANIFEST_LOAD,
        PermissionSlug::MANIFEST_UNLOAD,
        PermissionSlug::METRICS_READ,
        PermissionSlug::PERMISSION_LIST,
        PermissionSlug::PRODUCT_CREATE,
        PermissionSlug::PRODUCT_DELETE,
        PermissionSlug::PRODUCT_READ,
        PermissionSlug::PRODUCT_UPDATE,
        PermissionSlug::ROLE_CREATE,
        PermissionSlug::ROLE_LIST,
        PermissionSlug::ROLE_UPDATE_PERMISSIONS,
        PermissionSlug::USER_CHANGE_PASSWORD,
        PermissionSlug::USER_CREATE,
        PermissionSlug::USER_DELETE,
        PermissionSlug::USER_GET,
        PermissionSlug::USER_LIST,
        PermissionSlug::USER_UPDATE,
        PermissionSlug::USER_UPDATE_ROLES,
    ];
}
