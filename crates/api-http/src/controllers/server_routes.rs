//! A rota de estado do serviço.

use crate::controllers::server_controller::ServerController;
use crate::router::route::Route;

/// A tabela de estado do serviço.
pub(crate) fn routes<C: ServerController>(controller: C) -> Vec<Route> {
    vec![Route::get("/info", move || controller.clone().info())]
}
