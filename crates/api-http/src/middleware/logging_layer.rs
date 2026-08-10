//! O layer de log de requisição.

use portmaster_app::LoggerFactory;
use tower::Layer;

use super::logging::{Logging, CHANNEL};

/// Aplica o [`Logging`].
#[derive(Clone)]
pub(crate) struct LoggingLayer<F> {
    /// De onde o logger do canal HTTP sai.
    factory: F,
}

impl<F> LoggingLayer<F> {
    /// Monta o layer com o que o provider entregou.
    pub(crate) const fn new(factory: F) -> Self {
        Self { factory }
    }
}

impl<S, F: LoggerFactory> Layer<S> for LoggingLayer<F> {
    type Service = Logging<S, F::Instance>;

    fn layer(&self, inner: S) -> Self::Service {
        Logging {
            inner,
            logger: self.factory.create(CHANNEL),
        }
    }
}
