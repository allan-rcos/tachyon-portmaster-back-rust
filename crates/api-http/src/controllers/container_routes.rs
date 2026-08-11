//! As rotas de contêiner.
//!
//! `/containers/summary` vem **antes** de `/containers/{id}`: o axum casa o
//! segmento literal primeiro de qualquer forma, mas escrevê-los nesta ordem é o
//! que deixa a precedência visível para quem lê.

use crate::controllers::container_controller::ContainerController;
use crate::router::route::Route;

/// A tabela de contêiner.
pub(crate) fn routes<C: ContainerController>(controller: C) -> Vec<Route> {
    let list = controller.clone();
    let create = controller.clone();
    let summary = controller.clone();
    let read = controller.clone();
    let update = controller.clone();
    let delete = controller.clone();
    let seal = controller.clone();

    vec![
        Route::get("/containers", move |params| list.clone().list(params)),
        Route::post("/containers", move |body| create.clone().create(body)),
        Route::get("/containers/summary", move |params| {
            summary.clone().summary(params)
        }),
        Route::get("/containers/{id}", move |id| read.clone().get(id)),
        Route::put("/containers/{id}", move |id, body| {
            update.clone().update(id, body)
        }),
        Route::delete("/containers/{id}", move |id| delete.clone().delete(id)),
        Route::post("/containers/{id}/seal", move |id| seal.clone().seal(id)),
        Route::post("/containers/{id}/dispatch", move |id| {
            controller.clone().dispatch(id)
        }),
    ]
}
