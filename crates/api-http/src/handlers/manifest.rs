//! `/manifests` — embarcar e desembarcar carga.
//!
//! Duas rotas com o mesmo formato de corpo e a mesma resposta; o que muda é o
//! sentido do movimento. Não há `GET`: o manifesto de um contêiner sai pelo
//! resumo (`GET /containers/summary`), que é onde ele tem contexto.
//!
//! ## A resposta é o contêiner, não a linha movimentada
//!
//! Quem embarcou já sabe o que embarcou — foi ele quem pediu. O que ele não sabe
//! é o efeito: quanto o contêiner passou a pesar e se saiu de vazio, que é o que
//! decide se o próximo passo é carregar mais ou selar. A mensagem acompanha por
//! contrato do schema.

use portmaster_app::commands::manifest::MoveItemCommand;
use portmaster_app::services::ManifestUseCase;

use crate::error::api_error::ApiError;
use crate::session::Session;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::dto::container::container_response_factory::ContainerResponseFactory;
use crate::wire::dto::manifest::load_item_request_factory::LoadItemRequestFactory;
use crate::wire::dto::manifest::manifest_response_factory::ManifestResponseFactory;
use crate::wire::dto::manifest::unload_item_request_factory::UnloadItemRequestFactory;
use crate::wire::wire::Wire;

/// O que o embarque responde.
const LOADED: &str = "Item loaded successfully.";

/// O que o desembarque responde.
const UNLOADED: &str = "Item unloaded successfully.";

/// Os handlers de manifesto.
pub struct ManifestHandlers<M> {
    /// O caso de uso de manifesto.
    manifest: M,
}

impl<M: ManifestUseCase> ManifestHandlers<M> {
    /// Monta os handlers.
    pub(crate) const fn new(manifest: M) -> Self {
        Self { manifest }
    }

    /// `POST /manifests/load-item`
    pub(crate) async fn load(
        &self,
        wire: Wire,
        Body(request): Body<LoadItemRequestFactory>,
    ) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

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

        Ok(ApiResponse::ok(
            wire,
            ManifestResponseFactory::of(
                LOADED,
                ContainerResponseFactory::of_domain(container.as_ref()),
            ),
        ))
    }

    /// `POST /manifests/unload-item`
    pub(crate) async fn unload(
        &self,
        wire: Wire,
        Body(request): Body<UnloadItemRequestFactory>,
    ) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

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

        Ok(ApiResponse::ok(
            wire,
            ManifestResponseFactory::of(
                UNLOADED,
                ContainerResponseFactory::of_domain(container.as_ref()),
            ),
        ))
    }
}
