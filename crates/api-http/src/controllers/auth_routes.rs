//! As rotas de sessão.

use axum::http::HeaderMap;
use axum::routing::post;
use axum::Router;

use crate::controllers::auth_controller::AuthController;
use crate::error::api_error::ApiError;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::encoder::Encoder;
use crate::wire::no_content::NoContent;

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
            post(move |encoder: Encoder, Body(request)| async move {
                match setup.setup(request).await {
                    Ok((body, cookies)) => cookies.into_iter().fold(
                        ApiResponse::created(encoder, Ok(body)),
                        ApiResponse::with_cookie,
                    ),
                    Err(error) => ApiResponse::created(encoder, Err(error)),
                }
            }),
        )
        .route(
            "/auth/login",
            post(move |encoder: Encoder, Body(request)| async move {
                match login.login(request).await {
                    Ok((body, cookies)) => cookies
                        .into_iter()
                        .fold(ApiResponse::ok(encoder, Ok(body)), ApiResponse::with_cookie),
                    Err(error) => ApiResponse::ok(encoder, Err(error)),
                }
            }),
        )
        .route(
            "/auth/refresh",
            post(move |encoder: Encoder, headers: HeaderMap| async move {
                refresh
                    .refresh(headers)
                    .await
                    .map(|cookies| cookies.into_iter().fold(NoContent::new(), NoContent::with_cookie))
                    .map_err(|error: ApiError| error.with_encoder(encoder))
            }),
        )
        .route(
            "/auth/logout",
            post(move |headers: HeaderMap| async move {
                controller
                    .logout(headers)
                    .await
                    .into_iter()
                    .fold(NoContent::new(), NoContent::with_cookie)
            }),
        )
}
