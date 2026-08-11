//! As rotas de carga.

use axum::routing::post;
use axum::Router;

use crate::controllers::manifest_controller::ManifestController;
use crate::session::Session;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;

/// Liga os handlers de carga aos caminhos.
pub(crate) fn routes<C: ManifestController>(controller: C) -> Router {
    let load = controller.clone();

    Router::new()
        .route(
            "/manifests/load-item",
            post(move |Body(request)| async move {
                ApiResponse::ok(
                    async {
                        let context = Session::require_user()?;
                        load.load(context, request).await
                    }
                    .await,
                )
            }),
        )
        .route(
            "/manifests/unload-item",
            post(move |Body(request)| async move {
                ApiResponse::ok(
                    async {
                        let context = Session::require_user()?;
                        controller.unload(context, request).await
                    }
                    .await,
                )
            }),
        )
}
