//! As rotas de contêiner.

use axum::routing::{get, post};
use axum::Router;

use crate::controllers::container_controller::ContainerController;

/// Liga os handlers de contêiner aos caminhos.
///
/// `/containers/summary` vem **antes** de `/containers/{id}`: o axum casa o
/// segmento literal primeiro, mas escrevê-los nesta ordem é o que deixa a
/// precedência visível para quem lê.
pub(crate) fn routes<C: ContainerController>(controller: C) -> Router {
    let list = controller.clone();
    let create = controller.clone();
    let summary = controller.clone();
    let read = controller.clone();
    let update = controller.clone();
    let delete = controller.clone();
    let seal = controller.clone();

    Router::new()
        .route(
            "/containers",
            post(move |body| create.clone().create(body))
                .get(move |params| list.clone().list(params)),
        )
        .route(
            "/containers/summary",
            get(move |params| summary.clone().summary(params)),
        )
        .route(
            "/containers/{id}",
            get(move |id| read.clone().get(id))
                .put(move |id, body| update.clone().update(id, body))
                .delete(move |id| delete.clone().delete(id)),
        )
        .route(
            "/containers/{id}/seal",
            post(move |id| seal.clone().seal(id)),
        )
        .route(
            "/containers/{id}/dispatch",
            post(move |id| controller.clone().dispatch(id)),
        )
}
