//! Ler o painel do pátio.

use crate::context::UserContext;

/// Ler o painel.
#[derive(Debug, Clone)]
pub struct GetMetricsQuery {
    /// Quem está consultando.
    pub context: UserContext,
}
