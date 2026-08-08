//! A camada que decide o formato de cada requisição.

use std::sync::Arc;

use tower::Layer;

use crate::middleware::negotiation::Negotiation;
use crate::wire::strategy::encode_strategy::EncodeStrategy;
use crate::wire::strategy::flatbuffers_encode_strategy::FlatBuffersEncodeStrategy;
use crate::wire::strategy::json_encode_strategy::JsonEncodeStrategy;

/// Instala a negociação de conteúdo na pilha.
///
/// Os dois `Arc` nascem **aqui, uma vez, no boot** — as strategies são ZSTs, e o
/// que cada requisição carrega é um clone de ponteiro. Construí-los por
/// requisição seria uma alocação por chamada para guardar nada.
///
/// **Esta camada é load-bearing:** toda rota que extrai um
/// [`Wire`](crate::wire::wire::Wire) responde 500 se ela não estiver na pilha.
/// É deliberado — ver o `FromRequestParts` do `Wire`.
#[derive(Clone)]
pub struct NegotiationLayer {
    json: Arc<dyn EncodeStrategy>,
    flatbuffers: Arc<dyn EncodeStrategy>,
}

impl NegotiationLayer {
    /// Monta a camada com as duas strategies de saída.
    pub fn new() -> Self {
        Self {
            json: Arc::new(JsonEncodeStrategy),
            flatbuffers: Arc::new(FlatBuffersEncodeStrategy),
        }
    }
}

impl Default for NegotiationLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for NegotiationLayer {
    type Service = Negotiation<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Negotiation::new(inner, self.json.clone(), self.flatbuffers.clone())
    }
}
