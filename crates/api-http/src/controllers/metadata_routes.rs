//! As rotas de metadado de sistema.

use axum::routing::get;
use axum::Router;

use crate::controllers::metadata_controller::MetadataController;

/// Liga os handlers de metadado aos caminhos.
pub(crate) fn routes<C: MetadataController>(controller: C) -> Router {
    Router::new().route(
        "/metadata/permissions",
        get(move |params| controller.clone().list_permissions(params)),
    )
}
