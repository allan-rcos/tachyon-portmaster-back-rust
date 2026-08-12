//! O formato de saída da tarefa corrente, e o Strategy que ele escolhe.
//!
//! > **Lacuna conhecida do `lint-exports`:** o `CURRENT` abaixo nasce de
//! > `tokio::task_local!`, e itens gerados por macro são invisíveis ao `syn`.
//! > O arquivo exporta dois itens na prática, não um.

use std::future::Future;

use axum::http::{header, StatusCode};
use axum::response::{IntoResponse as _, Response};

use crate::middleware::encode_port::EncodePort;
use crate::wire::media_type::MediaType;
use crate::wire::strategy::encode_strategy::EncodeStrategy as _;
use crate::wire::strategy::flatbuffers_encode_strategy::FlatBuffersEncodeStrategy;
use crate::wire::strategy::json_encode_strategy::JsonEncodeStrategy;
use crate::wire::x::response_x::ResponseX;

tokio::task_local! {
    /// O formato que esta requisição negociou para a resposta.
    static CURRENT: MediaType;
}

/// O contexto do Strategy pattern na saída.
///
/// O estado do contexto é a variante guardada no escopo, e as strategies são
/// ZSTs imóveis: trocar de formato é trocar a variante, não realocar nada. Não
/// há enum de strategy à parte porque não precisa haver — a
/// [`MediaType`] **é** o seletor, e um segundo enum espelhando-a só criaria a
/// chance de os dois discordarem.
///
/// ZST: o formato é da tarefa, não do objeto. Injetá-lo custa o mesmo que não
/// injetar.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct EncodeContext;

impl EncodeContext {
    /// Roda `future` com este formato instalado na tarefa.
    ///
    /// `pub(super)`: é o escritor, e só o layer irmão o alcança. De fora não há
    /// como um handler responder num formato que a requisição não pediu.
    pub(super) async fn scope<F: Future>(media: MediaType, future: F) -> F::Output {
        CURRENT.scope(media, future).await
    }

    /// Abre o escopo de fora do módulo, só em teste.
    ///
    /// O escritor de verdade é `pub(super)` e continua sendo: em produção só o
    /// layer irmão escolhe o formato. Este é o seam que deixa um teste de outro
    /// módulo — o do [`ApiError`](crate::ports::error::api_error::ApiError), que
    /// precisa provar que o erro sai no formato negociado — montar o escopo sem
    /// subir a pilha inteira.
    #[cfg(test)]
    pub(crate) async fn scope_for_test<F: Future>(media: MediaType, future: F) -> F::Output {
        Self::scope(media, future).await
    }

    /// O formato desta tarefa, ou JSON.
    ///
    /// Cair em JSON fora do escopo é deliberado, e é o oposto do que a sessão
    /// faz. Um erro que nasce **antes** do middleware — o pânico que o `Recover`
    /// pega numa camada mais externa, um teste que exercita um controller solto
    /// — ainda precisa virar resposta, e recusar aqui seria responder que não
    /// sabemos responder, o que exigiria escolher um formato para dizê-lo.
    fn current() -> MediaType {
        CURRENT.try_with(|media| *media).unwrap_or(MediaType::Json)
    }
}

impl EncodePort for EncodeContext {
    fn respond<X: ResponseX>(&self, status: StatusCode, body: &X) -> Response {
        let media = Self::current();

        let encoded = match media {
            MediaType::Json => JsonEncodeStrategy.encode(body),
            MediaType::FlatBuffers => FlatBuffersEncodeStrategy.encode(body),
        };

        match encoded {
            Ok(bytes) => (
                status,
                [(header::CONTENT_TYPE, media.header_value())],
                bytes,
            )
                .into_response(),
            Err(error) => error.into_response(),
        }
    }
}

#[cfg(test)]
#[path = "tests/encode_context_test.rs"]
mod tests;
