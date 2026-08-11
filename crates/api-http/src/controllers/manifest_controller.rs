//! O contrato do controller de carga.

use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::vo::manifest::load_item_x_request::LoadItemXRequest;
use crate::wire::vo::manifest::manifest_x_response::ManifestXResponse;
use crate::wire::vo::manifest::unload_item_x_request::UnloadItemXRequest;

/// Os handlers de carga.
#[trait_variant::make(Send)]
pub(crate) trait ManifestController: Clone + Sync + 'static {
    /// `POST /manifests/load-item`
    async fn load(self, request: Body<LoadItemXRequest>) -> ApiResponse<ManifestXResponse>;

    /// `POST /manifests/unload-item`
    async fn unload(self, request: Body<UnloadItemXRequest>) -> ApiResponse<ManifestXResponse>;
}
