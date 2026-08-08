//! Ler um contêiner.

use crate::context::UserContext;

/// Ler um contêiner.
#[derive(Debug, Clone)]
pub struct GetContainerQuery {
    /// Quem está consultando.
    pub context: UserContext,
    /// Id do contêiner, em base62.
    pub id: String,
}
