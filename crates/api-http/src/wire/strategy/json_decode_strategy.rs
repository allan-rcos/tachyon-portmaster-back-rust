//! A leitura de um corpo JSON.

use crate::ports::error::api_error::ApiError;
use crate::wire::strategy::decode_strategy::DecodeStrategy;
use crate::wire::x::request_x::RequestX;

/// Lê o corpo como o DTO de JSON da mensagem.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct JsonDecodeStrategy;

impl DecodeStrategy for JsonDecodeStrategy {
    /// Desserializa direto no DTO da mensagem.
    ///
    /// O `derive` do serde gera este código no build; antes o corpo virava um
    /// `Map<String, Value>` e cada factory pescava campo a campo em tempo de
    /// execução, sem que o compilador soubesse de nada. O tipo agora é o
    /// contrato, e um campo renomeado no DTO quebra a compilação em vez de
    /// virar `None` silencioso.
    ///
    /// A mensagem do serde nomeia linha e coluna, e devolvê-la ajuda quem está
    /// integrando — não há segredo nela, o corpo é o que o cliente mandou.
    fn decode<X: RequestX>(&self, bytes: &[u8]) -> Result<X, ApiError> {
        let dto: X::Json = serde_json::from_slice(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo JSON inválido: {e}")))?;

        Ok(X::of_json(dto))
    }
}
