//! A rota de estado do serviço.

use axum::routing::get;
use axum::Router;

use crate::controllers::server_controller::ServerController;

/// Liga o handler de `/info` ao caminho.
pub(crate) fn routes<C: ServerController>(controller: C) -> Router {
    Router::new().route("/info", get(move || controller.clone().info()))
}
