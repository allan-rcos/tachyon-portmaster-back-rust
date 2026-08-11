//! O contrato do controller de metadados de sistema.

use portmaster_app::context::UserContext;

use crate::controllers::params::search_params::SearchParams;
use crate::ports::error::api_error::ApiError;
use crate::wire::vo::metadata::permission_list_x_response::PermissionListXResponse;

/// Os handlers de metadado de sistema.
#[trait_variant::make(Send)]
pub(crate) trait MetadataController: Clone + Sync + 'static {
    /// `GET /metadata/permissions`
    async fn list_permissions(
        &self,
        context: UserContext,
        params: SearchParams,
    ) -> Result<PermissionListXResponse, ApiError>;
}
