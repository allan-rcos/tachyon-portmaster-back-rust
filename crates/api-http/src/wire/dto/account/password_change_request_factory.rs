//! Como ler um [`PasswordChangeRequest`] dos dois formatos.

use planus::ReadAsRoot as _;
use serde_json::{Map, Value};

use crate::error::api_error::ApiError;
use crate::wire::dto::account::password_change_request::PasswordChangeRequest;
use crate::wire::factory::request_factory::RequestFactory;
use crate::wire::json::Json;
use crate::wire::tables as fbs;

/// Lê a troca da própria senha.
pub(crate) struct PasswordChangeRequestFactory;

impl RequestFactory for PasswordChangeRequestFactory {
    type Message = PasswordChangeRequest;

    fn from_json(source: &Map<String, Value>) -> Result<Self::Message, ApiError> {
        Ok(PasswordChangeRequest {
            current_password: Json::text(source, "current_password"),
            new_password: Json::text(source, "new_password"),
        })
    }

    fn from_flatbuffer(bytes: &[u8]) -> Result<Self::Message, ApiError> {
        let table = fbs::account::AccountPasswordChangeRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(PasswordChangeRequest {
            current_password: table.current_password().ok().map(str::to_owned),
            new_password: table.new_password().ok().map(str::to_owned),
        })
    }
}
