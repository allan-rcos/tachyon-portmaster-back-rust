//! Como ler um [`UserCreateRequest`] dos dois formatos.

use planus::ReadAsRoot as _;
use serde_json::{Map, Value};

use crate::error::api_error::ApiError;
use crate::wire::dto::admin::user_create_request::UserCreateRequest;
use crate::wire::factory::request_factory::RequestFactory;
use crate::wire::json::Json;
use crate::wire::tables as fbs;

/// Lê a criação de um usuário.
pub(crate) struct UserCreateRequestFactory;

impl RequestFactory for UserCreateRequestFactory {
    type Message = UserCreateRequest;

    fn from_json(source: &Map<String, Value>) -> Result<Self::Message, ApiError> {
        Ok(UserCreateRequest {
            name: Json::text(source, "name"),
            email: Json::text(source, "email"),
            initial_password: Json::text(source, "initial_password"),
            role_ids: Json::texts(source, "role_ids"),
        })
    }

    fn from_flatbuffer(bytes: &[u8]) -> Result<Self::Message, ApiError> {
        let table = fbs::admin::UserCreateRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(UserCreateRequest {
            name: table.name().ok().map(str::to_owned),
            email: table.email().ok().map(str::to_owned),
            initial_password: table.initial_password().ok().map(str::to_owned),
            role_ids: table.role_ids().ok().flatten().map(|ids| {
                ids.into_iter()
                    .filter_map(|v| v.map(str::to_owned).ok())
                    .collect()
            }),
        })
    }
}
