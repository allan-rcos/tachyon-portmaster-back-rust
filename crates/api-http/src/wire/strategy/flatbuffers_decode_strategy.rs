//! A leitura de um corpo `FlatBuffers`.

use crate::ports::error::api_error::ApiError;
use crate::wire::strategy::decode_strategy::DecodeStrategy;
use crate::wire::x::request_x::RequestX;

/// Entrega os bytes crus ao VO, que sabe qual tabela ler.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct FlatBuffersDecodeStrategy;

impl DecodeStrategy for FlatBuffersDecodeStrategy {
    fn decode<X: RequestX>(&self, bytes: &[u8]) -> Result<X, ApiError> {
        X::of_fbs(bytes)
    }
}

#[cfg(test)]
#[path = "tests/flatbuffers_decode_strategy_test.rs"]
mod tests;
