//! As rotas do painel.

use crate::controllers::metrics_controller::MetricsController;
use crate::router::route::Route;

/// A tabela do painel.
pub(crate) fn routes<C: MetricsController>(controller: C) -> Vec<Route> {
    vec![Route::get("/metrics", move || controller.clone().get())]
}
