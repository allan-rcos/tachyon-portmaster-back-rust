//! As rotas de papel.

use crate::controllers::role_controller::RoleController;
use crate::router::route::Route;

/// A tabela de papel.
pub(crate) fn routes<C: RoleController>(controller: C) -> Vec<Route> {
    let list = controller.clone();
    let create = controller.clone();

    vec![
        Route::get("/roles", move |params| list.clone().list(params)),
        Route::post("/roles", move |body| create.clone().create(body)),
        Route::put("/roles/{id}/permissions", move |id, body| {
            controller.clone().update_permissions(id, body)
        }),
    ]
}
