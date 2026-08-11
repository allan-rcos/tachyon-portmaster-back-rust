//! O contexto do Strategy pattern na entrada.

use crate::ports::error::api_error::ApiError;
use crate::wire::media_type::MediaType;
use crate::wire::strategy::decode_strategy::DecodeStrategy as _;
use crate::wire::strategy::flatbuffers_decode_strategy::FlatBuffersDecodeStrategy;
use crate::wire::strategy::json_decode_strategy::JsonDecodeStrategy;
use crate::wire::x::request_x::RequestX;

/// A strategy da vez.
///
/// Privado ao módulo pela mesma razão do seu par na saída: quem decodifica não
/// descobre de qual formato veio o corpo.
#[derive(Debug, Clone, Copy)]
enum Strategy {
    /// Lê JSON.
    Json(JsonDecodeStrategy),
    /// Lê `FlatBuffers`.
    FlatBuffers(FlatBuffersDecodeStrategy),
}

/// Quem lê o corpo no formato que a requisição anunciou.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Decoder {
    /// A strategy em uso.
    current: Strategy,
}

impl Decoder {
    /// O decoder para o que o `Content-Type` anunciou.
    pub(crate) fn of_request(content_type: Option<&str>) -> Self {
        let mut decoder = Self {
            current: Strategy::FlatBuffers(FlatBuffersDecodeStrategy),
        };
        decoder.set(MediaType::of_request(content_type));

        decoder
    }

    /// Troca a strategy corrente.
    pub(crate) const fn set(&mut self, media: MediaType) {
        self.current = match media {
            MediaType::Json => Strategy::Json(JsonDecodeStrategy),
            MediaType::FlatBuffers => Strategy::FlatBuffers(FlatBuffersDecodeStrategy),
        };
    }

    /// Lê o VO do corpo.
    pub(crate) fn decode<X: RequestX>(self, bytes: &[u8]) -> Result<X, ApiError> {
        match self.current {
            Strategy::Json(strategy) => strategy.decode(bytes),
            Strategy::FlatBuffers(strategy) => strategy.decode(bytes),
        }
    }
}
