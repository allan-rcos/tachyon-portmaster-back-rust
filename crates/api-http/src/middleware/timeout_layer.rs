//! O layer de teto de tempo.

use std::time::Duration;

use tower::Layer;

use super::timeout::Timeout;

/// Aplica o [`Timeout`].
#[derive(Clone, Copy)]
pub(crate) struct TimeoutLayer {
    /// Teto de tempo ou de tamanho, conforme o tipo.
    limit: Duration,
}

impl TimeoutLayer {
    /// Monta o layer com o teto configurado.
    pub(crate) const fn new(limit: Duration) -> Self {
        Self { limit }
    }
}

impl<S> Layer<S> for TimeoutLayer {
    type Service = Timeout<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Timeout {
            inner,
            limit: self.limit,
        }
    }
}
