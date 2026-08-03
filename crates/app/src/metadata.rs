//! Os metadados de sistema — hoje, as permissões registradas.
//!
//! O catálogo é preenchido no boot e imutável depois disso. Não há caso de uso
//! de escrita aqui: registrar é trabalho do [`crate::register`], e uma permissão
//! criada em runtime seria uma que nenhum caso de uso exige.

use portmaster_infra::repository::PermissionRepository;

use crate::authorization::{slug, RequiresPermission};
use crate::context::UserContext;
use crate::error::AppError;

/// Listar as permissões registradas.
#[derive(Debug, Clone)]
pub struct ListPermissionsQuery {
    /// Quem está consultando.
    pub context: UserContext,
    /// Filtra por trecho do slug.
    pub search: Option<String>,
}

/// O que a apresentação pode pedir sobre metadados.
#[trait_variant::make(Send)]
pub trait MetadataUseCase {
    /// Os slugs registrados, em ordem.
    async fn list_permissions(&self, query: ListPermissionsQuery) -> Result<Vec<String>, AppError>;
}

/// A implementação, genérica sobre os ports que consome.
pub(crate) struct MetadataUseCaseImpl<R> {
    permissions: R,
    list_permission: RequiresPermission,
}

impl<R> MetadataUseCaseImpl<R> {
    /// Monta o caso de uso, declarando a permissão que ele exige.
    pub(crate) fn new(permissions: R) -> Self {
        Self {
            permissions,
            list_permission: RequiresPermission::new(slug::PERMISSION_LIST),
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
