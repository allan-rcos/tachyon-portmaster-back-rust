//! As rotas da própria conta.

use crate::controllers::account_controller::AccountController;
use crate::router::route::Route;

/// A tabela da própria conta.
pub(crate) fn routes<C: AccountController>(controller: C) -> Vec<Route> {
    let read = controller.clone();
    let update = controller.clone();

    vec![
        Route::get("/account", move || read.clone().get()),
        Route::put("/account", move |body| update.clone().update(body)),
        Route::put("/account/password", move |body| {
            controller.clone().change_password(body)
        }),
    ]
}
