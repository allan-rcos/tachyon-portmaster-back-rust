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

use portmaster_app::domain::Container;
use portmaster_app::manifest::{ManifestUseCase, MoveItemCommand};

use crate::error::{app_error_to_status, ApiError};
use crate::session::Session;
use crate::wire::http::{Accept, Body, Negotiated};
use crate::wire::tables as fbs;
use crate::wire::view::container_of;

/// O que o embarque responde.
const LOADED: &str = "Item loaded successfully.";

/// O que o desembarque responde.
const UNLOADED: &str = "Item unloaded successfully.";

/// Os handlers de manifesto.
pub(crate) struct ManifestHandlers<M> {
    manifest: M,
}

impl<M: ManifestUseCase> ManifestHandlers<M> {
    /// Monta os handlers.
    pub(crate) fn new(manifest: M) -> Self {
        Self { manifest }
    }

    /// `POST /manifests/load-item`
    pub(crate) async fn load(
        &self,
        accept: Accept,
        Body(request): Body<fbs::manifest::LoadItemRequest>,
    ) -> Result<Negotiated<fbs::manifest::ManifestResponse>, ApiError> {
        let context = Session::require_user()?;

        let container = self
            .manifest
            .load(MoveItemCommand {
                context,
                container_id: request.container_id.unwrap_or_default(),
                product_id: request.product_id.unwrap_or_default(),
                quantity: request.quantity,
            })
            .await
            .map_err(app_error_to_status)?;

        Ok(Negotiated::ok(accept, response(LOADED, container.as_ref())))
    }

    /// `POST /manifests/unload-item`
    pub(crate) async fn unload(
        &self,
        accept: Accept,
        Body(request): Body<fbs::manifest::UnloadItemRequest>,
    ) -> Result<Negotiated<fbs::manifest::ManifestResponse>, ApiError> {
        let context = Session::require_user()?;

        let container = self
            .manifest
            .unload(MoveItemCommand {
                context,
                container_id: request.container_id.unwrap_or_default(),
                product_id: request.product_id.unwrap_or_default(),
                quantity: request.quantity,
            })
            .await
            .map_err(app_error_to_status)?;

        Ok(Negotiated::ok(
            accept,
            response(UNLOADED, container.as_ref()),
        ))
    }
}

/// Monta a resposta de um movimento.
fn response(message: &str, container: &dyn Container) -> fbs::manifest::ManifestResponse {
    fbs::manifest::ManifestResponse {
        message: Some(message.to_owned()),
        container: Some(Box::new(container_of(container))),
    }
}
