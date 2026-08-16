//! A orquestração de metadados.
//!
//! ## As permissões são privadas
//!
//! O slug abaixo é **contrato**: já existe em papéis gravados no banco de quem
//! roda a versão PHP, e renomeá-lo revoga silenciosamente o acesso de quem o
//! tinha. É `const` privada porque uma permissão pertence a exatamente um caso
//! de uso — é ele quem a compara com o `UserContext`, e não há segundo lugar no
//! sistema que precise vê-la.

use portmaster_domain::table_modules::PermissionTM;
use portmaster_infra::repository::PermissionRepository;

use crate::commands::metadata::RegisterPermissionCommand;
use crate::error::{AppError, MetadataError};
use crate::queries::metadata::ListPermissionsQuery;
use crate::services::MetadataService;

/// Listar as permissões registradas.
const PERMISSION_LIST: &str = "permission:list";

/// Monta o caso de uso de metadados.
///
/// Os ports chegam injetados e o que sai é o contrato: o tipo concreto não tem
/// nome fora deste arquivo, então nada além do provider consegue depender do
/// formato dele.
pub(crate) fn metadata_service<R, T>(
    permissions: R,
    permission_tm: T,
) -> impl MetadataService + Sync + Clone + use<R, T> + 'static
where
    R: PermissionRepository + Send + Sync + Clone + 'static,
    T: PermissionTM + Send + Sync + Clone + 'static,
{
    MetadataServiceImpl {
        permissions,
        permission_tm,
    }
}

/// A implementação, genérica sobre os ports que consome.
#[derive(Clone)]
struct MetadataServiceImpl<R, T> {
    /// O catálogo de permissões, em memória.
    permissions: R,
    /// As regras de permissão — quem confere o formato do slug.
    permission_tm: T,
}

impl<R, T> MetadataService for MetadataServiceImpl<R, T>
where
    R: PermissionRepository + Send + Sync,
    T: PermissionTM + Send + Sync,
{
    /// Registra a permissão que este serviço exige.
    ///
    /// Não recebe registrador: este **é** o registrador, e pedir a si mesmo por
    /// parâmetro só acrescentaria cerimônia.
    async fn declare_permissions(&self) -> Result<(), MetadataError> {
        self.register_permission(RegisterPermissionCommand {
            slug: PERMISSION_LIST.to_owned(),
        })
        .await
    }

    /// Registra uma permissão no catálogo.
    ///
    /// Sem transação: o registro **é** um cache em memória, e envolvê-lo numa
    /// transação abriria uma conexão de banco para não consultar banco nenhum.
    /// Sem checagem de permissão: quem chama é o boot, e não há chamador.
    async fn register_permission(
        &self,
        command: RegisterPermissionCommand,
    ) -> Result<(), MetadataError> {
        let permission = self.permission_tm.create(command.slug)?;

        self.permissions.register(permission.as_ref()).await?;

        Ok(())
    }

    /// O catálogo de permissões, opcionalmente filtrado.
    ///
    /// Sem transação e sem cache de leitura, pela mesma razão de
    /// [`Self::register_permission`]: são dezenas de entradas fixas em memória.
    async fn list_permissions(
        &self,
        query: ListPermissionsQuery,
    ) -> Result<Vec<String>, MetadataError> {
        if !query.context.has_permission(PERMISSION_LIST) {
            return Err(AppError::permission_denied(PERMISSION_LIST).into());
        }

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

#[cfg(test)]
#[path = "tests/metadata_service_impl_test.rs"]
mod tests;
