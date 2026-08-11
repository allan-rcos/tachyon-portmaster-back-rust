//! O contrato de quem lê o corpo no formato anunciado.

use crate::ports::error::api_error::ApiError;
use crate::wire::x::request_x::RequestX;

/// Lê um VO de requisição do formato que esta requisição anunciou.
///
/// Genérica sobre o VO pela mesma razão da
/// [`EncodePort`](crate::middleware::encode_port::EncodePort), e com a mesma
/// consequência: quem chama não descobre de qual formato o corpo veio.
///
/// Um formato que não sabemos ler nunca chega aqui — quem o recusa é o
/// middleware, com `415`, antes de o corpo ser lido.
pub(crate) trait DecodePort: Clone + Send + Sync + 'static {
    /// Desserializa o VO a partir dos bytes do corpo.
    fn decode<X: RequestX>(&self, bytes: &[u8]) -> Result<X, ApiError>;
}
