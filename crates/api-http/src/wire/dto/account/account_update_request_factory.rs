//! Como ler um [`AccountUpdateRequest`] dos dois formatos.

use planus::ReadAsRoot as _;
use serde_json::{Map, Value};

use crate::error::api_error::ApiError;
use crate::wire::dto::account::account_update_request::AccountUpdateRequest;
use crate::wire::factory::request_factory::RequestFactory;
use crate::wire::json::Json;
use crate::wire::tables as fbs;

/// Lê a alteração do próprio perfil.
pub(crate) struct AccountUpdateRequestFactory;

impl RequestFactory for AccountUpdateRequestFactory {
    type Message = AccountUpdateRequest;

    fn from_json(source: &Map<String, Value>) -> Result<Self::Message, ApiError> {
        Ok(AccountUpdateRequest {
            name: Json::text(source, "name"),
            email: Json::text(source, "email"),
        })
    }

    fn from_flatbuffer(bytes: &[u8]) -> Result<Self::Message, ApiError> {
        let table = fbs::account::AccountUpdateRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(AccountUpdateRequest {
            name: table.name().ok().map(str::to_owned),
            email: table.email().ok().map(str::to_owned),
        })
    }
}
