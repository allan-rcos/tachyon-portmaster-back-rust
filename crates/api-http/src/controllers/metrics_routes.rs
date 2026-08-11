//! As rotas do painel.

use axum::routing::get;
use axum::Router;

use crate::controllers::metrics_controller::MetricsController;
use crate::session::Session;
use crate::wire::api_response::ApiResponse;

/// Liga os handlers do painel aos caminhos.
pub(crate) fn routes<C: MetricsController>(controller: C) -> Router {
    Router::new().route(
        "/metrics",
        get(move || async move {
            ApiResponse::ok(
                async {
                    let context = Session::require_user()?;
                    controller.get(context).await
                }
                .await,
            )
        }),
    )
}
