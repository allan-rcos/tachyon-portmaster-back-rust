//! As rotas de contêiner.

use axum::extract::{Path, Query};
use axum::routing::{get, post};
use axum::Router;

use crate::controllers::container_controller::ContainerController;
use crate::controllers::params::container_page_params::ContainerPageParams;
use crate::controllers::params::summary_page_params::SummaryPageParams;
use crate::middleware::intern::session_context::SessionContext;
use crate::middleware::session_port::SessionPort as _;
use crate::ports::error::api_error::ApiError;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;

/// Liga os handlers de contêiner aos caminhos.
///
/// `/containers/summary` é declarada **antes** de `/containers/{id}`: o axum
/// casa a rota mais específica primeiro, mas deixar a ordem explícita evita que
/// uma reordenação futura faça `summary` virar um id.
///
/// Dividida em duas metades porque o contêiner é o único recurso com oito
/// rotas: a coleção de um lado, o item e as duas transições de estado do outro.
pub(crate) fn routes<C: ContainerController>(controller: C) -> Router {
    collection(controller.clone()).merge(item(controller))
}

/// As rotas que falam do conjunto: listar, resumir e criar.
fn collection<C: ContainerController>(controller: C) -> Router {
    let list = controller.clone();
    let summary = controller.clone();
    let create = controller;

    Router::new()
        .route(
            "/containers",
            post(move |Body(request)| async move {
                ApiResponse::created(
                    async {
                        let context = SessionContext.require_user()?;
                        create.create(context, request).await
                    }
                    .await,
                )
            })
            .get(
                move |Query(params): Query<ContainerPageParams>| async move {
                    ApiResponse::ok(
                        async {
                            let context = SessionContext.require_user()?;
                            list.list(context, params).await
                        }
                        .await,
                    )
                },
            ),
        )
        .route(
            "/containers/summary",
            get(move |Query(params): Query<SummaryPageParams>| async move {
                ApiResponse::ok(
                    async {
                        let context = SessionContext.require_user()?;
                        summary.summary(context, params).await
                    }
                    .await,
                )
            }),
        )
}

/// As rotas que falam de um contêiner: ler, alterar, apagar, selar e despachar.
fn item<C: ContainerController>(controller: C) -> Router {
    let read = controller.clone();
    let update = controller.clone();
    let delete = controller.clone();
    let seal = controller.clone();

    Router::new()
        .route(
            "/containers/{id}",
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
            "/containers/{id}/seal",
            post(move |Path(id): Path<String>| async move {
                async {
                    let context = SessionContext.require_user()?;
                    seal.seal(context, id).await?;

                    Ok::<_, ApiError>(ApiResponse::no_content())
                }
                .await
            }),
        )
        .route(
            "/containers/{id}/dispatch",
            post(move |Path(id): Path<String>| async move {
                async {
                    let context = SessionContext.require_user()?;
                    controller.dispatch(context, id).await?;

                    Ok::<_, ApiError>(ApiResponse::no_content())
                }
                .await
            }),
        )
}
