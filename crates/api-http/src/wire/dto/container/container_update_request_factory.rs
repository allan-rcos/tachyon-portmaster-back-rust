//! Como ler um [`ContainerUpdateRequest`] dos dois formatos.

use planus::ReadAsRoot as _;
use serde_json::{Map, Value};

use crate::error::api_error::ApiError;
use crate::wire::dto::container::container_update_request::ContainerUpdateRequest;
use crate::wire::factory::request_factory::RequestFactory;
use crate::wire::json::Json;
use crate::wire::tables as fbs;

/// Lê a alteração de um contêiner.
pub(crate) struct ContainerUpdateRequestFactory;

impl RequestFactory for ContainerUpdateRequestFactory {
    type Message = ContainerUpdateRequest;

    fn from_json(source: &Map<String, Value>) -> Result<Self::Message, ApiError> {
        Ok(ContainerUpdateRequest {
            max_capacity: Json::real(source, "max_capacity"),
        })
    }

    fn from_flatbuffer(bytes: &[u8]) -> Result<Self::Message, ApiError> {
        let table = fbs::container::ContainerUpdateRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(ContainerUpdateRequest {
            max_capacity: table.max_capacity().ok(),
        })
    }
}
