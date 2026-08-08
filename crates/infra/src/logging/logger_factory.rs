//! O contrato da fábrica de loggers.

use crate::logging::Logger;

/// Cria loggers nomeados.
///
/// O nome identifica a origem — `auth`, `http`, `container` — e vira campo em
/// toda linha que aquele logger emitir.
pub trait LoggerFactory: Clone + Send + Sync + 'static {
    /// Um logger para o componente indicado.
    fn create(&self, name: &str) -> Logger;
}
