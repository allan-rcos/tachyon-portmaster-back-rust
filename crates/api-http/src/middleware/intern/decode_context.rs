//! O formato de entrada da tarefa corrente, e o Strategy que ele escolhe.
//!
//! > **Lacuna conhecida do `lint-exports`:** o `CURRENT` abaixo nasce de
//! > `tokio::task_local!`, e itens gerados por macro são invisíveis ao `syn`.
//! > O arquivo exporta dois itens na prática, não um.

use std::future::Future;

use crate::middleware::decode_port::DecodePort;
use crate::ports::error::api_error::ApiError;
use crate::wire::media_type::MediaType;
use crate::wire::strategy::decode_strategy::DecodeStrategy as _;
use crate::wire::strategy::flatbuffers_decode_strategy::FlatBuffersDecodeStrategy;
use crate::wire::strategy::json_decode_strategy::JsonDecodeStrategy;
use crate::wire::x::request_x::RequestX;

tokio::task_local! {
    /// O formato que esta requisição anunciou para o corpo.
    static CURRENT: MediaType;
}

/// O contexto do Strategy pattern na entrada.
///
/// Par exato do [`EncodeContext`](super::encode_context::EncodeContext), e
/// separado dele de propósito: `Content-Type` e `Accept` são cabeçalhos
/// independentes, e um cliente que manda `FlatBuffers` e pede JSON de volta é um
/// caso normal — não uma inconsistência a resolver.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct DecodeContext;

impl DecodeContext {
    /// Roda `future` com este formato instalado na tarefa.
    ///
    /// `pub(super)`: é o escritor, e só o layer irmão o alcança.
    pub(super) async fn scope<F: Future>(media: MediaType, future: F) -> F::Output {
        CURRENT.scope(media, future).await
    }

    /// O formato desta tarefa, ou o nativo.
    ///
    /// Fora do escopo vale `FlatBuffers`, que é o mesmo destino de um corpo sem
    /// `Content-Type`: quem manda bytes sem anunciar o tipo é um cliente nosso
    /// falando o formato nativo.
    fn current() -> MediaType {
        CURRENT
            .try_with(|media| *media)
            .unwrap_or(MediaType::FlatBuffers)
    }
}

impl DecodePort for DecodeContext {
    fn decode<X: RequestX>(&self, bytes: &[u8]) -> Result<X, ApiError> {
        match Self::current() {
            MediaType::Json => JsonDecodeStrategy.decode(bytes),
            MediaType::FlatBuffers => FlatBuffersDecodeStrategy.decode(bytes),
        }
    }
}

#[cfg(test)]
#[path = "tests/decode_context_test.rs"]
mod tests;
