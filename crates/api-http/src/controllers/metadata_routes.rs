//! As rotas de metadado de sistema.

use axum::extract::Query;
use axum::routing::get;
use axum::Router;

use crate::controllers::metadata_controller::MetadataController;
use crate::controllers::params::search_params::SearchParams;
use crate::session::Session;
use crate::wire::api_response::ApiResponse;
use crate::wire::encoder::Encoder;

/// Liga os handlers de metadado aos caminhos.
pub(crate) fn routes<C: MetadataController>(controller: C) -> Router {
    Router::new().route(
        "/metadata/permissions",
        get(
            move |encoder: Encoder, Query(params): Query<SearchParams>| async move {
                ApiResponse::ok(
                    encoder,
                    async {
                        let context = Session::require_user()?;
                        controller.list_permissions(context, params).await
                    }
                    .await,
                )
            },
        ),
    )
}
