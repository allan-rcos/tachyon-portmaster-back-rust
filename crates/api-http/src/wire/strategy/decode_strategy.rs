//! O contrato de quem lê um corpo, sem saber que mensagem é.

use crate::error::api_error::ApiError;
use crate::wire::factory::request_factory::RequestFactory;

/// Lê um corpo no formato desta strategy.
///
/// **Não é object-safe, e isso é o desenho.** O `decode` é genérico sobre a
/// factory, então a trait não vira `dyn` — o que mantém o caminho de requisição
/// inteiramente estático, monomorfizado por rota. Quem escolhe a strategy é um
/// `match` sobre o [`MediaType`](crate::wire::media_type::MediaType), não uma
/// vTable.
///
/// A assimetria com a [`EncodeStrategy`](super::encode_strategy::EncodeStrategy)
/// é proposital: só a saída precisa ser carregada por valor até o fim da
/// requisição, e por isso só ela paga um `Arc<dyn>`.
pub(crate) trait DecodeStrategy {
    /// Lê a mensagem que a factory descreve.
    fn decode<F: RequestFactory>(&self, bytes: &[u8]) -> Result<F::Message, ApiError>;
}
