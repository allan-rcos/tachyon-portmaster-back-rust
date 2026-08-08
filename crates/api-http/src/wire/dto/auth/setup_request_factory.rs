//! Como ler um [`SetupRequest`] dos dois formatos.

use planus::ReadAsRoot as _;
use serde_json::{Map, Value};

use crate::error::api_error::ApiError;
use crate::wire::dto::auth::setup_request::SetupRequest;
use crate::wire::factory::request_factory::RequestFactory;
use crate::wire::json::Json;
use crate::wire::tables as fbs;

/// Lê os dados do primeiro usuário.
pub(crate) struct SetupRequestFactory;

impl RequestFactory for SetupRequestFactory {
    type Message = SetupRequest;

    fn from_json(source: &Map<String, Value>) -> Result<Self::Message, ApiError> {
        Ok(SetupRequest {
            name: Json::text(source, "name"),
            email: Json::text(source, "email"),
            password: Json::text(source, "password"),
        })
    }

    fn from_flatbuffer(bytes: &[u8]) -> Result<Self::Message, ApiError> {
        let table = fbs::auth::SetupRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(SetupRequest {
            name: table.name().ok().map(str::to_owned),
            email: table.email().ok().map(str::to_owned),
            password: table.password().ok().map(str::to_owned),
        })
    }
}
