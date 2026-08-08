//! O que a requisição decidiu sobre formato.

use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;

use crate::error::api_error::ApiError;
use crate::wire::media_type::MediaType;
use crate::wire::strategy::encode_strategy::EncodeStrategy;

/// A negociação de conteúdo desta requisição.
///
/// É o gêmeo dos dois `*StrategyManager` do PHP, colapsado num tipo só, e
/// **assimétrico de propósito**: a saída carrega a strategy, a entrada carrega
/// só o `MediaType`. Não há o que guardar do lado da entrada, porque
/// [`DecodeStrategy`](crate::wire::strategy::decode_strategy::DecodeStrategy)
/// não é object-safe — o `decode` é genérico sobre a factory, e um `match`
/// resolve a escolha sem vTable.
///
/// ## Por que `Arc` e não o task-local do PHP
///
/// O PHP guardava a strategy num contexto de task da coroutine. Aqui ela viaja
/// nas `extensions` da requisição, que o axum já carrega e que o compilador já
/// garante que não vaza para outra. Não há coroutine para embaralhar, e o
/// `Arc` é clonado do ponteiro criado **no boot** — as strategies são ZSTs.
#[derive(Clone)]
pub(crate) struct Wire {
    /// O formato do corpo que chegou.
    request: MediaType,
    /// Como escrever a resposta.
    encode: Arc<dyn EncodeStrategy>,
}

impl Wire {
    /// Monta a negociação a partir do que os cabeçalhos disseram.
    pub(crate) const fn new(request: MediaType, encode: Arc<dyn EncodeStrategy>) -> Self {
        Self { request, encode }
    }

    /// O formato do corpo que chegou.
    pub(crate) const fn request(&self) -> MediaType {
        self.request
    }

    /// A strategy que escreve a resposta.
    pub(crate) fn encode(&self) -> &dyn EncodeStrategy {
        self.encode.as_ref()
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Wire {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Ausência aqui é layer fora de ordem, não requisição malformada: é
        // defeito nosso, e precisa explodir na primeira requisição em vez de
        // responder no formato errado em silêncio. Mesma decisão do
        // `Session::gate()`.
        parts.extensions.get::<Self>().cloned().ok_or_else(|| {
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "negociação de conteúdo ausente: o NegotiationLayer não está na pilha",
            )
        })
    }
}
