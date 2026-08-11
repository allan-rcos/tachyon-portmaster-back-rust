//! As rotas do painel.

use axum::routing::get;
use axum::Router;

use crate::controllers::metrics_controller::MetricsController;

/// Liga os handlers do painel aos caminhos.
pub(crate) fn routes<C: MetricsController>(controller: C) -> Router {
    Router::new().route("/metrics", get(move || controller.clone().get()))
}
