//! As rotas de produto.

use axum::extract::{Path, Query};
use axum::routing::{get, post};
use axum::Router;

use crate::controllers::params::page_params::PageParams;
use crate::controllers::product_controller::ProductController;
use crate::ports::error::api_error::ApiError;
use crate::session::Session;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;

/// Liga os handlers de produto aos caminhos.
pub(crate) fn routes<C: ProductController>(controller: C) -> Router {
    let list = controller.clone();
    let create = controller.clone();
    let read = controller.clone();
    let update = controller.clone();

    Router::new()
        .route(
            "/products",
            post(move |Body(request)| async move {
                ApiResponse::created(
                    async {
                        let context = Session::require_user()?;
                        create.create(context, request).await
                    }
                    .await,
                )
            })
            .get(move |Query(params): Query<PageParams>| async move {
                ApiResponse::ok(
                    async {
                        let context = Session::require_user()?;
                        list.list(context, params).await
                    }
                    .await,
                )
            }),
        )
        .route(
            "/products/{id}",
            get(move |Path(id): Path<String>| async move {
                ApiResponse::ok(
                    async {
                        let context = Session::require_user()?;
                        read.get(context, id).await
                    }
                    .await,
                )
            })
            .put(move |Path(id): Path<String>, Body(request)| async move {
                ApiResponse::ok(
                    async {
                        let context = Session::require_user()?;
                        update.update(context, id, request).await
                    }
                    .await,
                )
            })
            .delete(move |Path(id): Path<String>| async move {
                async {
                    let context = Session::require_user()?;
                    controller.delete(context, id).await?;

                    Ok::<_, ApiError>(ApiResponse::no_content())
                }
                .await
            }),
        )
}
