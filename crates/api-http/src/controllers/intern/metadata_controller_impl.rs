//! O controller de metadados de sistema. Não sai do módulo.

use portmaster_app::context::UserContext;
use portmaster_app::error::MetadataError;
use portmaster_app::queries::metadata::ListPermissionsQuery;
use portmaster_app::services::MetadataUseCase;

use crate::controllers::metadata_controller::MetadataController;
use crate::controllers::params::search_params::SearchParams;
use crate::error::api_error::ApiError;
use crate::wire::vo::metadata::permission_list_x_response::PermissionListXResponse;

/// Os handlers de metadado, genéricos sobre o caso de uso.
#[derive(Clone)]
pub(crate) struct MetadataControllerImpl<M> {
    /// O caso de uso de metadado.
    metadata: M,
}

impl<M: MetadataUseCase> MetadataControllerImpl<M> {
    /// Monta o controller.
    pub(crate) const fn new(metadata: M) -> Self {
        Self { metadata }
    }
}

impl<M: MetadataUseCase + Clone + Send + Sync + 'static> MetadataController
    for MetadataControllerImpl<M>
{
    async fn list_permissions(
        &self,
        context: UserContext,
        params: SearchParams,
    ) -> Result<PermissionListXResponse, ApiError> {
        let slugs = self
            .metadata
            .list_permissions(ListPermissionsQuery {
                context,
                search: params.search,
            })
            .await
            .map_err(to_api)?;

        Ok(PermissionListXResponse::of(slugs))
    }
}

/// Traduz a recusa do serviço de metadados.
///
/// Ele só devolve o erro comum: a listagem não endereça recurso nenhum.
fn to_api(error: MetadataError) -> ApiError {
    match error {
        MetadataError::App(shared) => ApiError::of_app(shared),
    }
}
