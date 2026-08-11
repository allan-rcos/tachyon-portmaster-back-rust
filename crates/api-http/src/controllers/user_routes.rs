//! As rotas de usuário.

use axum::extract::{Path, Query};
use axum::routing::{get, post, put};
use axum::Router;

use crate::controllers::params::user_page_params::UserPageParams;
use crate::controllers::user_controller::UserController;
use crate::middleware::intern::session_context::SessionContext;
use crate::middleware::session_port::SessionPort as _;
use crate::ports::error::api_error::ApiError;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;

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
            post(move |Body(request)| async move {
                ApiResponse::created(
                    async {
                        let context = SessionContext.require_user()?;
                        create.create(context, request).await
                    }
                    .await,
                )
            })
            .get(move |Query(params): Query<UserPageParams>| async move {
                ApiResponse::ok(
                    async {
                        let context = SessionContext.require_user()?;
                        list.list(context, params).await
                    }
                    .await,
                )
            }),
        )
        .route(
            "/users/{id}",
            get(move |Path(id): Path<String>| async move {
                ApiResponse::ok(
                    async {
                        let context = SessionContext.require_user()?;
                        read.get(context, id).await
                    }
                    .await,
                )
            })
            .put(move |Path(id): Path<String>, Body(request)| async move {
                ApiResponse::ok(
                    async {
                        let context = SessionContext.require_user()?;
                        update.update(context, id, request).await
                    }
                    .await,
                )
            })
            .delete(move |Path(id): Path<String>| async move {
                async {
                    let context = SessionContext.require_user()?;
                    delete.delete(context, id).await?;

                    Ok::<_, ApiError>(ApiResponse::no_content())
                }
                .await
            }),
        )
        .route(
            "/users/{id}/roles",
            put(move |Path(id): Path<String>, Body(request)| async move {
                ApiResponse::ok(
                    async {
                        let context = SessionContext.require_user()?;
                        roles.update_roles(context, id, request).await
                    }
                    .await,
                )
            }),
        )
        .route(
            "/users/{id}/password",
            put(move |Path(id): Path<String>, Body(request)| async move {
                async {
                    let context = SessionContext.require_user()?;
                    controller.reset_password(context, id, request).await?;

                    Ok::<_, ApiError>(ApiResponse::no_content())
                }
                .await
            }),
        )
}
