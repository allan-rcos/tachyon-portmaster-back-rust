//! Como ler um [`ContainerCreateRequest`] dos dois formatos.

use planus::ReadAsRoot as _;
use serde_json::{Map, Value};

use crate::error::api_error::ApiError;
use crate::wire::dto::container::container_create_request::ContainerCreateRequest;
use crate::wire::factory::request_factory::RequestFactory;
use crate::wire::json::Json;
use crate::wire::tables as fbs;

/// Lê o registro de um contêiner.
pub(crate) struct ContainerCreateRequestFactory;

impl RequestFactory for ContainerCreateRequestFactory {
    type Message = ContainerCreateRequest;

    fn from_json(source: &Map<String, Value>) -> Result<Self::Message, ApiError> {
        Ok(ContainerCreateRequest {
            code: Json::text(source, "code"),
            max_capacity: Json::real(source, "max_capacity"),
        })
    }

    fn from_flatbuffer(bytes: &[u8]) -> Result<Self::Message, ApiError> {
        let table = fbs::container::ContainerCreateRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(ContainerCreateRequest {
            code: table.code().ok().flatten().map(str::to_owned),
            max_capacity: table.max_capacity().ok(),
        })
    }
}
