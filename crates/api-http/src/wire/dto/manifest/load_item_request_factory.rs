//! Como ler um [`LoadItemRequest`] dos dois formatos.

use planus::ReadAsRoot as _;
use serde_json::{Map, Value};

use crate::error::api_error::ApiError;
use crate::wire::dto::manifest::load_item_request::LoadItemRequest;
use crate::wire::factory::request_factory::RequestFactory;
use crate::wire::json::Json;
use crate::wire::tables as fbs;

/// Lê um embarque de carga.
pub(crate) struct LoadItemRequestFactory;

impl RequestFactory for LoadItemRequestFactory {
    type Message = LoadItemRequest;

    fn from_json(source: &Map<String, Value>) -> Result<Self::Message, ApiError> {
        Ok(LoadItemRequest {
            container_id: Json::text(source, "container_id"),
            product_id: Json::text(source, "product_id"),
            quantity: Json::real(source, "quantity"),
        })
    }

    fn from_flatbuffer(bytes: &[u8]) -> Result<Self::Message, ApiError> {
        let table = fbs::manifest::LoadItemRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(LoadItemRequest {
            container_id: table.container_id().ok().flatten().map(str::to_owned),
            product_id: table.product_id().ok().flatten().map(str::to_owned),
            quantity: table.quantity().ok(),
        })
    }
}
