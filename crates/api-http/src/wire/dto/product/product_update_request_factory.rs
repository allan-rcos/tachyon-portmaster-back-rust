//! Como ler um [`ProductUpdateRequest`] dos dois formatos.

use planus::ReadAsRoot as _;
use serde_json::{Map, Value};

use crate::error::api_error::ApiError;
use crate::wire::dto::product::product_update_request::ProductUpdateRequest;
use crate::wire::factory::request_factory::RequestFactory;
use crate::wire::json::Json;
use crate::wire::tables as fbs;

/// Lê a alteração de um produto.
pub(crate) struct ProductUpdateRequestFactory;

impl RequestFactory for ProductUpdateRequestFactory {
    type Message = ProductUpdateRequest;

    fn from_json(source: &Map<String, Value>) -> Result<Self::Message, ApiError> {
        Ok(ProductUpdateRequest {
            name: Json::text(source, "name"),
            density: Json::real(source, "density"),
            risk_class: Json::int(source, "risk_class"),
        })
    }

    fn from_flatbuffer(bytes: &[u8]) -> Result<Self::Message, ApiError> {
        let table = fbs::product::ProductUpdateRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(ProductUpdateRequest {
            name: table.name().ok().map(str::to_owned),
            density: table.density().ok(),
            risk_class: table.risk_class().ok().map(|v| i32::from(v as u8)),
        })
    }
}
