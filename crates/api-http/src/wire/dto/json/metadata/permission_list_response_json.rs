//! O DTO de JSON de `PermissionListResponse`.

use crate::wire::dto::json::metadata::metadata_item_response_json::MetadataItemResponseJson;
use serde::Serialize;

/// `PermissionListResponse` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct PermissionListResponseJson {
    /// As permissões que o sistema conhece.
    pub(crate) data: Vec<MetadataItemResponseJson>,
}
