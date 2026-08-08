//! `/metadata/permissions` — o catálogo do que um papel pode receber.
//!
//! O catálogo não é uma constante que o cliente possa ler do schema: ele é
//! preenchido no boot, a partir do que cada caso de uso declara exigir. Esta
//! rota é como um cliente descobre o conjunto — e ela mesma é guarda-costas,
//! porque saber o mapa de autorização já é privilégio.

use axum::extract::Query;
use portmaster_app::queries::metadata::ListPermissionsQuery;
use portmaster_app::services::MetadataUseCase;

use crate::error::api_error::ApiError;
use crate::handlers::params::search_params::SearchParams;
use crate::session::Session;
use crate::wire::api_response::ApiResponse;
use crate::wire::dto::metadata::permission_list_response_factory::PermissionListResponseFactory;
use crate::wire::wire::Wire;

/// Os handlers de metadado.
pub struct MetadataHandlers<M> {
    /// O caso de uso de metadado.
    metadata: M,
}

impl<M: MetadataUseCase> MetadataHandlers<M> {
    /// Monta os handlers.
    pub(crate) const fn new(metadata: M) -> Self {
        Self { metadata }
    }

    /// `GET /metadata/permissions`
    /// Sem correspondência é uma **lista vazia**, e não 404: o catálogo existe,
    /// e o que não existe é a busca — que é resposta, não ausência de recurso.
    pub(crate) async fn list_permissions(
        &self,
        wire: Wire,
        Query(params): Query<SearchParams>,
    ) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

        let slugs = self
            .metadata
            .list_permissions(ListPermissionsQuery {
                context,
                search: params.search,
            })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ApiResponse::ok(
            wire,
            PermissionListResponseFactory::of(slugs),
        ))
    }
}
