//! As rotas de carga.

use crate::controllers::manifest_controller::ManifestController;
use crate::router::route::Route;

/// A tabela de carga.
pub(crate) fn routes<C: ManifestController>(controller: C) -> Vec<Route> {
    let load = controller.clone();

    vec![
        Route::post("/manifests/load-item", move |body| load.clone().load(body)),
        Route::post("/manifests/unload-item", move |body| {
            controller.clone().unload(body)
        }),
    ]
}
