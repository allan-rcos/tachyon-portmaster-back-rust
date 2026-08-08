//! O contrato de quem escreve um corpo, sem saber que mensagem é.

use crate::error::api_error::ApiError;
use crate::wire::factory::renderable::Renderable;

/// Escreve uma resposta no formato desta strategy.
///
/// Object-safe de propósito: é a **única** coisa dinâmica do wire. O
/// [`Wire`](crate::wire::wire::Wire) carrega um `Arc<dyn EncodeStrategy>`
/// escolhido uma vez por requisição, no `NegotiationLayer`, e todo o resto —
/// leitura, montagem da tabela, aninhamento — continua estático.
///
/// Os dois `Arc` nascem **no boot**, não por requisição: as strategies são ZSTs,
/// e o que a requisição carrega é um clone de ponteiro.
pub(crate) trait EncodeStrategy: Send + Sync {
    /// Serializa a resposta.
    fn encode(&self, response: &dyn Renderable) -> Result<Vec<u8>, ApiError>;

    /// O valor de `Content-Type` que acompanha estes bytes.
    fn content_type(&self) -> &'static str;
}
