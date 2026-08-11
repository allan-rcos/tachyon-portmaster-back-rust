//! As rotas de papel.

use axum::routing::{post, put};
use axum::Router;

use crate::controllers::role_controller::RoleController;

/// Liga os handlers de papel aos caminhos.
pub(crate) fn routes<C: RoleController>(controller: C) -> Router {
    let list = controller.clone();
    let create = controller.clone();

    Router::new()
        .route(
            "/roles",
            post(move |body| create.clone().create(body))
                .get(move |params| list.clone().list(params)),
        )
        .route(
            "/roles/{id}/permissions",
            put(move |id, body| controller.clone().update_permissions(id, body)),
        )
}
