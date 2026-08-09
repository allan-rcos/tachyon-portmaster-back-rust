//! O contrato do controller de carga.

use portmaster_app::context::UserContext;

use crate::error::api_error::ApiError;
use crate::wire::vo::manifest::load_item_x_request::LoadItemXRequest;
use crate::wire::vo::manifest::manifest_x_response::ManifestXResponse;
use crate::wire::vo::manifest::unload_item_x_request::UnloadItemXRequest;

/// Os handlers de carga.
#[trait_variant::make(Send)]
pub(crate) trait ManifestController: Clone + Sync + 'static {
    /// `POST /manifests/load-item`
    async fn load(
        &self,
        context: UserContext,
        request: LoadItemXRequest,
    ) -> Result<ManifestXResponse, ApiError>;

    /// `POST /manifests/unload-item`
    async fn unload(
        &self,
        context: UserContext,
        request: UnloadItemXRequest,
    ) -> Result<ManifestXResponse, ApiError>;
}
