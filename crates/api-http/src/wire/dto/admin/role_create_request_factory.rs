//! Como ler um [`RoleCreateRequest`] dos dois formatos.

use planus::ReadAsRoot as _;
use serde_json::{Map, Value};

use crate::error::api_error::ApiError;
use crate::wire::dto::admin::role_create_request::RoleCreateRequest;
use crate::wire::factory::request_factory::RequestFactory;
use crate::wire::json::Json;
use crate::wire::tables as fbs;

/// Lê a criação de um papel.
pub(crate) struct RoleCreateRequestFactory;

impl RequestFactory for RoleCreateRequestFactory {
    type Message = RoleCreateRequest;

    fn from_json(source: &Map<String, Value>) -> Result<Self::Message, ApiError> {
        Ok(RoleCreateRequest {
            name: Json::text(source, "name"),
            permissions: Json::texts(source, "permissions"),
        })
    }

    fn from_flatbuffer(bytes: &[u8]) -> Result<Self::Message, ApiError> {
        let table = fbs::admin::RoleCreateRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(RoleCreateRequest {
            name: table.name().ok().map(str::to_owned),
            permissions: table.permissions().ok().flatten().map(|ids| {
                ids.into_iter()
                    .filter_map(|v| v.map(str::to_owned).ok())
                    .collect()
            }),
        })
    }
}
