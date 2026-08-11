//! As rotas de usuário.

use crate::controllers::user_controller::UserController;
use crate::router::route::Route;

/// A tabela de usuário.
pub(crate) fn routes<C: UserController>(controller: C) -> Vec<Route> {
    let list = controller.clone();
    let create = controller.clone();
    let read = controller.clone();
    let update = controller.clone();
    let delete = controller.clone();
    let roles = controller.clone();

    vec![
        Route::get("/users", move |params| list.clone().list(params)),
        Route::post("/users", move |body| create.clone().create(body)),
        Route::get("/users/{id}", move |id| read.clone().get(id)),
        Route::put("/users/{id}", move |id, body| {
            update.clone().update(id, body)
        }),
        Route::delete("/users/{id}", move |id| delete.clone().delete(id)),
        Route::put("/users/{id}/roles", move |id, body| {
            roles.clone().update_roles(id, body)
        }),
        Route::put("/users/{id}/password", move |id, body| {
            controller.clone().reset_password(id, body)
        }),
    ]
}
