//! Alterar um contêiner.

use crate::context::UserContext;

/// Alterar a capacidade de um contêiner.
#[derive(Debug, Clone)]
pub struct UpdateContainerCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Id do contêiner, em base62.
    pub id: String,
    /// Capacidade máxima.
    pub max_capacity: f64,
}
