//! A fábrica de loggers sobre o `tracing`.

use crate::logging::intern::tracing_logger::TracingLogger;
use crate::logging::LoggerFactory;

/// A fábrica de loggers sobre `tracing`.
#[derive(Debug, Clone, Default)]
pub(crate) struct TracingLoggerFactory;

impl TracingLoggerFactory {
    /// Monta a fábrica.
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl LoggerFactory for TracingLoggerFactory {
    type Instance = TracingLogger;

    fn create(&self, name: &str) -> Self::Instance {
        TracingLogger::new(name)
    }
}
