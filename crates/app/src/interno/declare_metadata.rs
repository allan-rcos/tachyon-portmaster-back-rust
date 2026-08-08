//! O registro de metadados que o boot faz uma vez.

use portmaster_domain::table_modules::{MarkerGroupTM, PermissionTM};
use portmaster_domain::DomainProvider;
use portmaster_infra::repository::{MarkerGroupRepository, PermissionRepository};
use portmaster_infra::InfraProvider;

use crate::security::permission_catalog::PermissionCatalog;
use crate::security::refresh_token_group::REFRESH_TOKEN_GROUP;

/// Preenche o catálogo de permissões e os grupos de marcador.
///
/// Falhar aqui derruba o boot de propósito: um sistema que subiu sem o catálogo
/// completo daria 403 em endpoints que deveriam funcionar, e a causa seria
/// invisível — o papel do administrador simplesmente não teria a permissão.
pub(crate) async fn declare_metadata<D: DomainProvider, I: InfraProvider>(
    domain: &D,
    infra: &I,
) -> anyhow::Result<()> {
    let permission_tm = domain.permission_table_module();
    let permissions = infra.permission_repository();

    for slug in PermissionCatalog::ALL {
        let permission = permission_tm.create((*slug).to_owned())?;
        permissions.register(permission.as_ref()).await?;
    }

    let group_tm = domain.marker_group_table_module();
    let groups = infra.marker_group_repository();
    let group = group_tm.create(REFRESH_TOKEN_GROUP.to_owned())?;
    groups.register(group.as_ref()).await?;

    Ok(())
}
