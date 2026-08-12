//! O controller de metadados de sistema. Não sai do módulo.

use portmaster_app::error::MetadataError;
use portmaster_app::queries::metadata::ListPermissionsQuery;
use portmaster_app::services::MetadataService;

use crate::controllers::metadata_controller::MetadataController;
use crate::controllers::params::search_params::SearchParams;
use crate::middleware::session_port::SessionPort;
use crate::ports::error::api_error::ApiError;
use crate::wire::api_response::ApiResponse;
use crate::wire::vo::metadata::permission_list_x_response::PermissionListXResponse;
use axum::extract::Query;

/// Os handlers de metadado, genéricos sobre o service.
#[derive(Clone)]
pub(crate) struct MetadataControllerImpl<M, S> {
    /// O service de metadado.
    metadata: M,
    /// Quem diz se há sessão, e quem a apresenta.
    session: S,
}

impl<M: MetadataService, S: SessionPort> MetadataControllerImpl<M, S> {
    /// Monta o controller.
    pub(crate) const fn new(metadata: M, session: S) -> Self {
        Self { metadata, session }
    }
}

impl<M: MetadataService + Clone + Send + Sync + 'static, S: SessionPort> MetadataController
    for MetadataControllerImpl<M, S>
{
    async fn list_permissions(
        self,
        Query(params): Query<SearchParams>,
    ) -> ApiResponse<PermissionListXResponse> {
        ApiResponse::ok(
            async {
                let context = self.session.require_user()?;

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
            .await,
        )
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
