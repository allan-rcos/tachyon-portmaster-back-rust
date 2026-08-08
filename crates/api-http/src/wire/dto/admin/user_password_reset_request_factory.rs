//! Como ler um [`UserPasswordResetRequest`] dos dois formatos.

use planus::ReadAsRoot as _;
use serde_json::{Map, Value};

use crate::error::api_error::ApiError;
use crate::wire::dto::admin::user_password_reset_request::UserPasswordResetRequest;
use crate::wire::factory::request_factory::RequestFactory;
use crate::wire::json::Json;
use crate::wire::tables as fbs;

/// Lê a redefinição de senha de um usuário.
pub(crate) struct UserPasswordResetRequestFactory;

impl RequestFactory for UserPasswordResetRequestFactory {
    type Message = UserPasswordResetRequest;

    fn from_json(source: &Map<String, Value>) -> Result<Self::Message, ApiError> {
        Ok(UserPasswordResetRequest {
            new_password: Json::text(source, "new_password"),
        })
    }

    fn from_flatbuffer(bytes: &[u8]) -> Result<Self::Message, ApiError> {
        let table = fbs::admin::UserAdminPasswordResetRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(UserPasswordResetRequest {
            new_password: table.new_password().ok().map(str::to_owned),
        })
    }
}
