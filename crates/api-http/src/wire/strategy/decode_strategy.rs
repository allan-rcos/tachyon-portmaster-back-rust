//! O contrato de quem lê um corpo de requisição.

use crate::ports::error::api_error::ApiError;
use crate::wire::x::request_x::RequestX;

/// Lê um VO de requisição de um formato.
///
/// Genérica sobre o VO pela mesma razão da
/// [`EncodeStrategy`](super::encode_strategy::EncodeStrategy), e com a mesma
/// consequência: quem chama não descobre qual formato a strategy lê.
pub(crate) trait DecodeStrategy {
    /// Desserializa o VO.
    fn decode<X: RequestX>(&self, bytes: &[u8]) -> Result<X, ApiError>;
}
