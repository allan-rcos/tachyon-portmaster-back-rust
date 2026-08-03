//! `/metadata/permissions` — o catálogo do que um papel pode receber.
//!
//! O catálogo não é uma constante que o cliente possa ler do schema: ele é
//! preenchido no boot, a partir do que cada caso de uso declara exigir. Esta
//! rota é como um cliente descobre o conjunto — e ela mesma é guarda-costas,
//! porque saber o mapa de autorização já é privilégio.

use axum::extract::Query;
use portmaster_app::metadata::{ListPermissionsQuery, MetadataUseCase};

use super::SearchParams;
use crate::error::{app_error_to_status, ApiError};
use crate::session::Session;
use crate::wire::http::{Accept, Negotiated};
use crate::wire::tables as fbs;
use crate::wire::view::permission_list;

/// Os handlers de metadado.
pub(crate) struct MetadataHandlers<M> {
    metadata: M,
}

impl<M: MetadataUseCase> MetadataHandlers<M> {
    /// Monta os handlers.
    pub(crate) fn new(metadata: M) -> Self {
        Self { metadata }
    }

    /// `GET /metadata/permissions`
    pub(crate) async fn list_permissions(
        &self,
        accept: Accept,
        Query(params): Query<SearchParams>,
    ) -> Result<Negotiated<fbs::metadata::PermissionListResponse>, ApiError> {
        let context = Session::require_user()?;

        let slugs = self
            .metadata
            .list_permissions(ListPermissionsQuery {
                context,
                search: params.search,
            })
            .await
            .map_err(app_error_to_status)?;

        // Sem correspondência é uma lista vazia, e não 404: o catálogo existe, e
        // o que não existe é a busca — que é resposta, não ausência de recurso.
        Ok(Negotiated::ok(accept, permission_list(slugs)))
    }
}
