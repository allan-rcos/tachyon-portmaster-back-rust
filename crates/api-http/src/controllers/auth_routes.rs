//! As rotas de sessão.
//!
//! As quatro são públicas no sentido de não exigirem access token — quem chega
//! em `/auth/refresh` está justamente com um que expirou. O que elas exigem,
//! cada uma à sua maneira, está dentro do controller.

use crate::controllers::auth_controller::AuthController;
use crate::router::route::Route;

/// A tabela de sessão.
pub(crate) fn routes<C: AuthController>(controller: C) -> Vec<Route> {
    let setup = controller.clone();
    let login = controller.clone();
    let refresh = controller.clone();

    vec![
        Route::post("/setup", move |body| setup.clone().setup(body)),
        Route::post("/auth/login", move |body| login.clone().login(body)),
        Route::post("/auth/refresh", move || refresh.clone().refresh()),
        Route::post("/auth/logout", move || controller.clone().logout()),
    ]
}
