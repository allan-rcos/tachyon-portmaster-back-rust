//! As rotas de produto.

use axum::routing::{get, post};
use axum::Router;

use crate::controllers::product_controller::ProductController;

/// Liga os handlers de produto aos caminhos.
pub(crate) fn routes<C: ProductController>(controller: C) -> Router {
    let list = controller.clone();
    let create = controller.clone();
    let read = controller.clone();
    let update = controller.clone();

    Router::new()
        .route(
            "/products",
            post(move |body| create.clone().create(body))
                .get(move |params| list.clone().list(params)),
        )
        .route(
            "/products/{id}",
            get(move |id| read.clone().get(id))
                .put(move |id, body| update.clone().update(id, body))
                .delete(move |id| controller.clone().delete(id)),
        )
}
