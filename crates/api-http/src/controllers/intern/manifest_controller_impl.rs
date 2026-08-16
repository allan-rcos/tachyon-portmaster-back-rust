//! O controller de carga. Não sai do módulo.

use axum::http::StatusCode;
use portmaster_app::commands::manifest::MoveItemCommand;
use portmaster_app::error::ManifestError;
use portmaster_app::services::ManifestService;

use crate::controllers::manifest_controller::ManifestController;
use crate::middleware::session_port::SessionPort;
use crate::ports::error::api_error::ApiError;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::vo::container::container_x_response::ContainerXResponse;
use crate::wire::vo::manifest::load_item_x_request::LoadItemXRequest;
use crate::wire::vo::manifest::manifest_x_response::ManifestXResponse;
use crate::wire::vo::manifest::unload_item_x_request::UnloadItemXRequest;

/// O que a resposta de embarque diz.
const LOADED: &str = "Item loaded successfully.";

/// O que a resposta de desembarque diz.
const UNLOADED: &str = "Item unloaded successfully.";

/// Monta o controller de manifesto.
///
/// O service e o acesso à sessão chegam injetados, e o que sai é o contrato: o
/// tipo concreto não tem nome fora deste arquivo.
pub(crate) fn manifest_controller<M, S>(
    manifest: M,
    session: S,
) -> impl ManifestController + use<M, S> + 'static
where
    M: ManifestService + Clone + Send + Sync + 'static,
    S: SessionPort + Clone + Send + Sync + 'static,
{
    ManifestControllerImpl { manifest, session }
}

/// Os handlers de carga, genéricos sobre o service.
#[derive(Clone)]
struct ManifestControllerImpl<M, S> {
    /// O service de carga.
    manifest: M,
    /// Quem diz se há sessão, e quem a apresenta.
    session: S,
}

impl<M: ManifestService + Clone + Send + Sync + 'static, S: SessionPort> ManifestController
    for ManifestControllerImpl<M, S>
{
    async fn load(self, Body(request): Body<LoadItemXRequest>) -> ApiResponse<ManifestXResponse> {
        ApiResponse::ok(
            async {
                let context = self.session.require_user()?;

                let container = self
                    .manifest
                    .load(MoveItemCommand {
                        context,
                        container_id: request.container_id.unwrap_or_default(),
                        product_id: request.product_id.unwrap_or_default(),
                        quantity: request.quantity.unwrap_or_default(),
                    })
                    .await
                    .map_err(to_api)?;

                Ok(ManifestXResponse {
                    message: LOADED.to_owned(),
                    container: ContainerXResponse::of_domain(container.as_ref()),
                })
            }
            .await,
        )
    }

    async fn unload(
        self,
        Body(request): Body<UnloadItemXRequest>,
    ) -> ApiResponse<ManifestXResponse> {
        ApiResponse::ok(
            async {
                let context = self.session.require_user()?;

                let container = self
                    .manifest
                    .unload(MoveItemCommand {
                        context,
                        container_id: request.container_id.unwrap_or_default(),
                        product_id: request.product_id.unwrap_or_default(),
                        quantity: request.quantity.unwrap_or_default(),
                    })
                    .await
                    .map_err(to_api)?;

                Ok(ManifestXResponse {
                    message: UNLOADED.to_owned(),
                    container: ContainerXResponse::of_domain(container.as_ref()),
                })
            }
            .await,
        )
    }
}

/// Traduz a recusa do serviço de manifesto no status que o cliente recebe.
///
/// Contêiner e produto ausentes são 404 cada um com o seu nome — os dois ids
/// chegam no mesmo corpo, e dizer só "não encontrado" deixaria o cliente sem
/// saber qual corrigir. Recusa de estado é 409.
fn to_api(error: ManifestError) -> ApiError {
    match error {
        ManifestError::MissingContainer(id) => ApiError::new(
            StatusCode::NOT_FOUND,
            format!("Container {id} was not found."),
        ),
        ManifestError::MissingProduct(id) => ApiError::new(
            StatusCode::NOT_FOUND,
            format!("Product {id} was not found."),
        ),
        ManifestError::Refused(refused) => ApiError::new(StatusCode::CONFLICT, refused.to_string()),
        ManifestError::App(shared) => ApiError::of_app(shared),
    }
}
