//! A escrita de uma resposta em `FlatBuffers`.

use crate::ports::error::api_error::ApiError;
use crate::wire::strategy::encode_strategy::EncodeStrategy;
use crate::wire::x::response_x::ResponseX;

/// Escreve a resposta como `FlatBuffers`.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct FlatBuffersEncodeStrategy;

impl EncodeStrategy for FlatBuffersEncodeStrategy {
    /// Escreve a tabela e devolve o buffer.
    ///
    /// O builder é descartado com o buffer: reaproveitá-lo entre requisições
    /// exigiria sincronizá-lo, e ele existe para durar uma serialização.
    ///
    /// Sem `Result` no caminho de erro real — o planus escreve a tabela que o
    /// tipo garante existir, e o aninhamento de filhos é coreografia dele.
    fn encode<X: ResponseX>(&self, response: &X) -> Result<Vec<u8>, ApiError> {
        let mut builder = planus::Builder::new();

        Ok(builder.finish(response.to_fbs(), None).to_vec())
    }
}
