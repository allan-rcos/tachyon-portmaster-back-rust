//! As rotas de papel.

use axum::extract::{Path, Query};
use axum::routing::{post, put};
use axum::Router;

use crate::controllers::params::page_params::PageParams;
use crate::controllers::role_controller::RoleController;
use crate::session::Session;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::encoder::Encoder;

/// Liga os handlers de papel aos caminhos.
///
/// Todo o encanamento de axum — extractor, status, negociação — mora aqui, ao
/// lado do controller que ele serve. O router de cima só faz `merge`, e por isso
/// não precisa nomear `Body<RolePermissionsUpdateXRequest>` nem saber que essa
/// rota tem corpo.
///
/// O controller entra por valor e é **clonado por requisição**: ele já foi
/// construído no boot, e o clone é de um punhado de handles.
pub(crate) fn routes<C: RoleController>(controller: C) -> Router {
    let list = controller.clone();
    let create = controller.clone();

    Router::new()
        .route(
            "/roles",
            post(move |encoder: Encoder, Body(request)| async move {
                let context = Session::require_user()?;

                Ok::<_, crate::error::api_error::ApiError>(ApiResponse::created(
                    encoder,
                    create.create(context, request).await,
                ))
            })
            .get(
                move |encoder: Encoder, Query(params): Query<PageParams>| async move {
                    let context = Session::require_user()?;

                    Ok::<_, crate::error::api_error::ApiError>(ApiResponse::ok(
                        encoder,
                        list.list(context, params).await,
                    ))
                },
            ),
        )
        .route(
            "/roles/{id}/permissions",
            put(
                move |encoder: Encoder, Path(id): Path<String>, Body(request)| async move {
                    let context = Session::require_user()?;

                    Ok::<_, crate::error::api_error::ApiError>(ApiResponse::ok(
                        encoder,
                        controller.update_permissions(context, id, request).await,
                    ))
                },
            ),
        )
}
