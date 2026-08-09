//! O controller de carga. Não sai do módulo.

use portmaster_app::commands::manifest::MoveItemCommand;
use portmaster_app::context::UserContext;
use portmaster_app::services::ManifestUseCase;

use crate::controllers::manifest_controller::ManifestController;
use crate::error::api_error::ApiError;
use crate::wire::vo::container::container_x_response::ContainerXResponse;
use crate::wire::vo::manifest::load_item_x_request::LoadItemXRequest;
use crate::wire::vo::manifest::manifest_x_response::ManifestXResponse;
use crate::wire::vo::manifest::unload_item_x_request::UnloadItemXRequest;

/// O que a resposta de embarque diz.
const LOADED: &str = "Item loaded successfully.";

/// O que a resposta de desembarque diz.
const UNLOADED: &str = "Item unloaded successfully.";

/// Os handlers de carga, genéricos sobre o caso de uso.
#[derive(Clone)]
pub(crate) struct ManifestControllerImpl<M> {
    /// O caso de uso de carga.
    manifest: M,
}

impl<M: ManifestUseCase> ManifestControllerImpl<M> {
    /// Monta o controller.
    pub(crate) const fn new(manifest: M) -> Self {
        Self { manifest }
    }
}

impl<M: ManifestUseCase + Clone + Send + Sync + 'static> ManifestController
    for ManifestControllerImpl<M>
{
    async fn load(
        &self,
        context: UserContext,
        request: LoadItemXRequest,
    ) -> Result<ManifestXResponse, ApiError> {
        let container = self
            .manifest
            .load(MoveItemCommand {
                context,
                container_id: request.container_id.unwrap_or_default(),
                product_id: request.product_id.unwrap_or_default(),
                quantity: request.quantity.unwrap_or_default(),
            })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ManifestXResponse {
            message: LOADED.to_owned(),
            container: ContainerXResponse::of_domain(container.as_ref()),
        })
    }

    async fn unload(
        &self,
        context: UserContext,
        request: UnloadItemXRequest,
    ) -> Result<ManifestXResponse, ApiError> {
        let container = self
            .manifest
            .unload(MoveItemCommand {
                context,
                container_id: request.container_id.unwrap_or_default(),
                product_id: request.product_id.unwrap_or_default(),
                quantity: request.quantity.unwrap_or_default(),
            })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ManifestXResponse {
            message: UNLOADED.to_owned(),
            container: ContainerXResponse::of_domain(container.as_ref()),
        })
    }
}
