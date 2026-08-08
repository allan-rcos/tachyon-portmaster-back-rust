//! Registrar um contêiner.

use crate::context::UserContext;

/// Registrar um contêiner.
#[derive(Debug, Clone)]
pub struct CreateContainerCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Código de identificação no pátio.
    pub code: String,
    /// Capacidade máxima.
    pub max_capacity: f64,
}
