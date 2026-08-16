//! Quem serve a fábrica de loggers.

use crate::logging::intern::tracing_logger_factory::tracing_logger_factory;
use crate::logging::LoggerFactory;

/// A fábrica de loggers.
///
/// Nada é guardado: a fábrica não tem estado — o nome é do logger, e o logger
/// nasce na chamada de `create`.
///
/// Quem não tem construtor onde injetar um logger não passa por aqui: usa o
/// [`SystemLogger`](crate::logging::SystemLogger), que é o global instalado no
/// boot.
pub(crate) struct LoggingProvider;

impl LoggingProvider {
    /// A fábrica de loggers nomeados.
    pub(crate) fn logger_factory() -> impl LoggerFactory + use<> {
        tracing_logger_factory()
    }
}
