//! A fábrica de loggers sobre o `tracing`.

use crate::logging::intern::tracing_logger::TracingLogger;
use crate::logging::LoggerFactory;

/// Monta a fábrica de loggers.
pub(crate) const fn tracing_logger_factory() -> impl LoggerFactory + use<> {
    TracingLoggerFactory
}

/// A fábrica de loggers sobre `tracing`.
#[derive(Debug, Clone, Default)]
struct TracingLoggerFactory;

impl LoggerFactory for TracingLoggerFactory {
    type Instance = TracingLogger;

    fn create(&self, name: &str) -> Self::Instance {
        TracingLogger::new(name)
    }
}
