//! Ler um produto.

use crate::context::UserContext;

/// Ler um produto.
#[derive(Debug, Clone)]
pub struct GetProductQuery {
    /// Quem está consultando.
    pub context: UserContext,
    /// Id do produto, em base62.
    pub id: String,
}
