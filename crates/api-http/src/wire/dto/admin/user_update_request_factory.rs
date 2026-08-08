//! Como ler um [`UserUpdateRequest`] dos dois formatos.

use planus::ReadAsRoot as _;
use serde_json::{Map, Value};

use crate::error::api_error::ApiError;
use crate::wire::dto::admin::user_update_request::UserUpdateRequest;
use crate::wire::factory::request_factory::RequestFactory;
use crate::wire::json::Json;
use crate::wire::tables as fbs;

/// Lê a alteração de um usuário.
pub(crate) struct UserUpdateRequestFactory;

impl RequestFactory for UserUpdateRequestFactory {
    type Message = UserUpdateRequest;

    fn from_json(source: &Map<String, Value>) -> Result<Self::Message, ApiError> {
        Ok(UserUpdateRequest {
            name: Json::text(source, "name"),
            email: Json::text(source, "email"),
        })
    }

    fn from_flatbuffer(bytes: &[u8]) -> Result<Self::Message, ApiError> {
        let table = fbs::admin::UserUpdateRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(UserUpdateRequest {
            name: table.name().ok().map(str::to_owned),
            email: table.email().ok().map(str::to_owned),
        })
    }
}
