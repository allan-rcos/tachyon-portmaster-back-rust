//! O layer do identificador de correlação.

use tower::Layer;

use super::request_id::RequestId;

/// Aplica o [`RequestId`].
#[derive(Clone)]
pub struct RequestIdLayer<G> {
    generator: G,
}

impl<G> RequestIdLayer<G> {
    /// Monta o layer com o gerador que o provider entregou.
    pub(crate) const fn new(generator: G) -> Self {
        Self { generator }
    }
}

impl<S, G: Clone> Layer<S> for RequestIdLayer<G> {
    type Service = RequestId<S, G>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestId {
            inner,
            generator: self.generator.clone(),
        }
    }
}
