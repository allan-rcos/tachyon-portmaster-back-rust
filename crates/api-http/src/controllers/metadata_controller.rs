//! O contrato do controller de metadados de sistema.

use crate::controllers::params::search_params::SearchParams;
use crate::wire::api_response::ApiResponse;
use crate::wire::vo::metadata::permission_list_x_response::PermissionListXResponse;
use axum::extract::Query;

/// Os handlers de metadado de sistema.
#[trait_variant::make(Send)]
pub(crate) trait MetadataController: Clone + Sync + 'static {
    /// `GET /metadata/permissions`
    async fn list_permissions(
        self,
        params: Query<SearchParams>,
    ) -> ApiResponse<PermissionListXResponse>;
}
