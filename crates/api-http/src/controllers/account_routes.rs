//! As rotas da própria conta.

use axum::routing::{get, put};
use axum::Router;

use crate::controllers::account_controller::AccountController;

/// Liga os handlers de conta aos caminhos.
pub(crate) fn routes<C: AccountController>(controller: C) -> Router {
    let read = controller.clone();
    let update = controller.clone();

    Router::new()
        .route(
            "/account",
            get(move || read.clone().get()).put(move |body| update.clone().update(body)),
        )
        .route(
            "/account/password",
            put(move |body| controller.clone().change_password(body)),
        )
}
