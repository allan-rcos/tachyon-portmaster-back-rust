//! As rotas da própria conta.

use axum::routing::{get, put};
use axum::Router;

use crate::controllers::account_controller::AccountController;
use crate::middleware::intern::session_context::SessionContext;
use crate::middleware::session_port::SessionPort as _;
use crate::ports::error::api_error::ApiError;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;

/// Liga os handlers de conta aos caminhos.
pub(crate) fn routes<C: AccountController>(controller: C) -> Router {
    let read = controller.clone();
    let update = controller.clone();

    Router::new()
        .route(
            "/account",
            get(move || async move {
                ApiResponse::ok(
                    async {
                        let context = SessionContext.require_user()?;
                        read.get(context).await
                    }
                    .await,
                )
            })
            .put(move |Body(request)| async move {
                ApiResponse::ok(
                    async {
                        let context = SessionContext.require_user()?;
                        update.update(context, request).await
                    }
                    .await,
                )
            }),
        )
        .route(
            "/account/password",
            put(move |Body(request)| async move {
                async {
                    let context = SessionContext.require_user()?;
                    controller.change_password(context, request).await?;

                    Ok::<_, ApiError>(ApiResponse::no_content())
                }
                .await
            }),
        )
}
