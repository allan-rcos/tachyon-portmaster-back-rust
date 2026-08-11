//! As rotas de sessão.

use axum::routing::post;
use axum::Router;

use crate::controllers::auth_controller::AuthController;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;

/// Liga os handlers de sessão aos caminhos.
///
/// As quatro rotas são públicas no sentido de não exigirem access token — quem
/// chega em `/auth/refresh` está justamente com um que expirou. O que elas
/// exigem, cada uma à sua maneira, está dentro do controller.
pub(crate) fn routes<C: AuthController>(controller: C) -> Router {
    let setup = controller.clone();
    let login = controller.clone();
    let refresh = controller.clone();

    Router::new()
        .route(
            "/setup",
            post(move |Body(request)| async move {
                ApiResponse::created(setup.setup(request).await)
            }),
        )
        .route(
            "/auth/login",
            post(move |Body(request)| async move { ApiResponse::ok(login.login(request).await) }),
        )
        .route(
            "/auth/refresh",
            post(move || async move { ApiResponse::no_content(refresh.refresh().await) }),
        )
        .route(
            "/auth/logout",
            post(move || async move { ApiResponse::no_content(controller.logout().await) }),
        )
}
