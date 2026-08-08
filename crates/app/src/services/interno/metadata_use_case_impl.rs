//! A orquestração de metadados.

use crate::error::AppError;
use crate::queries::metadata::ListPermissionsQuery;
use crate::security::requires_permission::RequiresPermission;
use crate::security::PermissionSlug;
use crate::services::MetadataUseCase;
use portmaster_infra::repository::PermissionRepository;

/// A implementação, genérica sobre os ports que consome.
pub(crate) struct MetadataUseCaseImpl<R> {
    permissions: R,
    list_permission: RequiresPermission,
}

impl<R> MetadataUseCaseImpl<R> {
    /// Monta o caso de uso, declarando a permissão que ele exige.
    pub(crate) const fn new(permissions: R) -> Self {
        Self {
            permissions,
            list_permission: RequiresPermission::new(PermissionSlug::PERMISSION_LIST),
        }
    }
}

impl<R: PermissionRepository + Send + Sync> MetadataUseCase for MetadataUseCaseImpl<R> {
    async fn list_permissions(&self, query: ListPermissionsQuery) -> Result<Vec<String>, AppError> {
        self.list_permission.authorize(&query.context)?;

        // Sem transação e sem cache de leitura: o registro **é** um cache em
        // memória, com dezenas de entradas fixas. Envolvê-lo numa transação
        // abriria uma conexão de banco para não consultar banco nenhum.
        let all = self.permissions.all().await?;

        let Some(needle) = query
            .search
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        else {
            return Ok(all);
        };

        let needle = needle.to_lowercase();

        Ok(all
            .into_iter()
            .filter(|slug| slug.to_lowercase().contains(&needle))
            .collect())
    }
}
