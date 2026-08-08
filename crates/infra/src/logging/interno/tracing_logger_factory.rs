//! A fábrica de loggers sobre o `tracing`.

use crate::logging::{Logger, LoggerFactory};

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
    fn create(&self, name: &str) -> Logger {
        Logger::new(name)
    }
}
