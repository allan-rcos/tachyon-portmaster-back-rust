//! As rotas de produto.

use crate::controllers::product_controller::ProductController;
use crate::router::route::Route;

/// A tabela de produto.
pub(crate) fn routes<C: ProductController>(controller: C) -> Vec<Route> {
    let list = controller.clone();
    let create = controller.clone();
    let read = controller.clone();
    let update = controller.clone();

    vec![
        Route::get("/products", move |params| list.clone().list(params)),
        Route::post("/products", move |body| create.clone().create(body)),
        Route::get("/products/{id}", move |id| read.clone().get(id)),
        Route::put("/products/{id}", move |id, body| {
            update.clone().update(id, body)
        }),
        Route::delete("/products/{id}", move |id| controller.clone().delete(id)),
    ]
}
