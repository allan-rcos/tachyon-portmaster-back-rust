//! As rotas de carga.

use axum::routing::post;
use axum::Router;

use crate::controllers::manifest_controller::ManifestController;

/// Liga os handlers de carga aos caminhos.
pub(crate) fn routes<C: ManifestController>(controller: C) -> Router {
    let load = controller.clone();

    Router::new()
        .route(
            "/manifests/load-item",
            post(move |body| load.clone().load(body)),
        )
        .route(
            "/manifests/unload-item",
            post(move |body| controller.clone().unload(body)),
        )
}
