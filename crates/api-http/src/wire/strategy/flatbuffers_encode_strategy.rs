//! A escrita de uma resposta em `FlatBuffers`.

use crate::error::api_error::ApiError;
use crate::wire::factory::renderable::Renderable;
use crate::wire::media_type::MediaType;
use crate::wire::strategy::encode_strategy::EncodeStrategy;

/// Escreve a resposta como `FlatBuffers`.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct FlatBuffersEncodeStrategy;

impl EncodeStrategy for FlatBuffersEncodeStrategy {
    /// O builder é descartado com o buffer: reaproveitá-lo entre requisições
    /// exigiria sincronizá-lo, e ele existe para durar uma serialização.
    fn encode(&self, response: &dyn Renderable) -> Result<Vec<u8>, ApiError> {
        let mut builder = planus::Builder::new();
        let root = response.write_flatbuffer(&mut builder)?;

        Ok(builder.finish(root, None).to_vec())
    }

    fn content_type(&self) -> &'static str {
        MediaType::FlatBuffers.header_value()
    }
}
