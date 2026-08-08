//! O layer de log de requisição.

use portmaster_app::LoggerFactory;
use tower::Layer;

use super::logging::{Logging, CHANNEL};

/// Aplica o [`Logging`].
#[derive(Clone)]
pub struct LoggingLayer<F> {
    factory: F,
}

impl<F> LoggingLayer<F> {
    /// Monta o layer com a fábrica que o provider entregou.
    pub(crate) const fn new(factory: F) -> Self {
        Self { factory }
    }
}

impl<S, F: LoggerFactory> Layer<S> for LoggingLayer<F> {
    type Service = Logging<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Logging {
            inner,
            logger: self.factory.create(CHANNEL),
        }
    }
}
