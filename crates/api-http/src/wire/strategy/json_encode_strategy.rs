//! A escrita de uma resposta em JSON.

use crate::error::api_error::ApiError;
use crate::wire::factory::renderable::Renderable;
use crate::wire::media_type::MediaType;
use crate::wire::strategy::encode_strategy::EncodeStrategy;

/// Escreve a resposta como JSON.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct JsonEncodeStrategy;

impl EncodeStrategy for JsonEncodeStrategy {
    fn encode(&self, response: &dyn Renderable) -> Result<Vec<u8>, ApiError> {
        let mut out = Vec::with_capacity(256);
        response.write_json(&mut out)?;

        Ok(out)
    }

    fn content_type(&self) -> &'static str {
        MediaType::Json.header_value()
    }
}
