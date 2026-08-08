//! Como ler um [`RolePermissionsUpdateRequest`] dos dois formatos.

use planus::ReadAsRoot as _;
use serde_json::{Map, Value};

use crate::error::api_error::ApiError;
use crate::wire::dto::admin::role_permissions_update_request::RolePermissionsUpdateRequest;
use crate::wire::factory::request_factory::RequestFactory;
use crate::wire::json::Json;
use crate::wire::tables as fbs;

/// Lê a troca de permissões de um papel.
pub(crate) struct RolePermissionsUpdateRequestFactory;

impl RequestFactory for RolePermissionsUpdateRequestFactory {
    type Message = RolePermissionsUpdateRequest;

    fn from_json(source: &Map<String, Value>) -> Result<Self::Message, ApiError> {
        Ok(RolePermissionsUpdateRequest {
            permissions: Json::texts(source, "permissions"),
        })
    }

    fn from_flatbuffer(bytes: &[u8]) -> Result<Self::Message, ApiError> {
        let table = fbs::admin::RolePermissionsUpdateRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(RolePermissionsUpdateRequest {
            permissions: table.permissions().ok().flatten().map(|ids| {
                ids.into_iter()
                    .filter_map(|v| v.map(str::to_owned).ok())
                    .collect()
            }),
        })
    }
}
