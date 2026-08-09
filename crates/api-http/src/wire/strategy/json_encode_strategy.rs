//! A escrita de uma resposta em JSON.

use crate::error::api_error::ApiError;
use crate::wire::strategy::encode_strategy::EncodeStrategy;
use crate::wire::x::response_x::ResponseX;

/// Escreve a resposta como JSON.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct JsonEncodeStrategy;

impl EncodeStrategy for JsonEncodeStrategy {
    /// Serializa o **DTO de JSON**, e não a tabela do planus.
    ///
    /// A ordem dos campos do corpo é a ordem em que o DTO os declara, e é ela
    /// que `swagger/swagger.json` documenta. Um `serde_json::Value` montado à
    /// mão não serviria: o `serde_json` deste lock não tem `preserve_order`, e
    /// os campos sairiam em ordem alfabética.
    fn encode<X: ResponseX>(&self, response: &X) -> Result<Vec<u8>, ApiError> {
        serde_json::to_vec(&response.to_json())
            .map_err(|e| ApiError::unrenderable(format!("falha ao escrever JSON: {e}")))
    }
}
