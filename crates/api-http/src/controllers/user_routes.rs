//! As rotas de usuário.

use axum::routing::{get, post, put};
use axum::Router;

use crate::controllers::user_controller::UserController;

/// Liga os handlers de usuário aos caminhos.
pub(crate) fn routes<C: UserController>(controller: C) -> Router {
    let list = controller.clone();
    let create = controller.clone();
    let read = controller.clone();
    let update = controller.clone();
    let delete = controller.clone();
    let roles = controller.clone();

    Router::new()
        .route(
            "/users",
            post(move |body| create.clone().create(body))
                .get(move |params| list.clone().list(params)),
        )
        .route(
            "/users/{id}",
            get(move |id| read.clone().get(id))
                .put(move |id, body| update.clone().update(id, body))
                .delete(move |id| delete.clone().delete(id)),
        )
        .route(
            "/users/{id}/roles",
            put(move |id, body| roles.clone().update_roles(id, body)),
        )
        .route(
            "/users/{id}/password",
            put(move |id, body| controller.clone().reset_password(id, body)),
        )
}
