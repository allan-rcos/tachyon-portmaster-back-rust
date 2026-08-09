//! O layer de log de requisição.

use portmaster_app::{Clock, LoggerFactory};
use tower::Layer;

use super::logging::{Logging, CHANNEL};

/// Aplica o [`Logging`].
#[derive(Clone)]
pub(crate) struct LoggingLayer<F, K> {
    /// De onde o logger de cada requisição sai.
    factory: F,
    /// De onde a latência sai.
    clock: K,
}

impl<F, K> LoggingLayer<F, K> {
    /// Monta o layer com o que o provider entregou.
    pub(crate) const fn new(factory: F, clock: K) -> Self {
        Self { factory, clock }
    }
}

impl<S, F: LoggerFactory, K: Clock> Layer<S> for LoggingLayer<F, K> {
    type Service = Logging<S, F::Instance, K>;

    fn layer(&self, inner: S) -> Self::Service {
        Logging {
            inner,
            logger: self.factory.create(CHANNEL),
            clock: self.clock.clone(),
        }
    }
}
