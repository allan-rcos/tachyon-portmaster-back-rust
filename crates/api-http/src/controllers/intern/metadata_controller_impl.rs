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

/// Monta o controller de metadados.
///
/// O service e o acesso à sessão chegam injetados, e o que sai é o contrato: o
/// tipo concreto não tem nome fora deste arquivo.
pub(crate) fn metadata_controller<M, S>(
    metadata: M,
    session: S,
) -> impl MetadataController + use<M, S> + 'static
where
    M: MetadataService + Clone + Send + Sync + 'static,
    S: SessionPort + Clone + Send + Sync + 'static,
{
    MetadataControllerImpl { metadata, session }
}

/// Os handlers de metadado, genéricos sobre o service.
#[derive(Clone)]
struct MetadataControllerImpl<M, S> {
    /// O service de metadado.
    metadata: M,
    /// Quem diz se há sessão, e quem a apresenta.
    session: S,
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
