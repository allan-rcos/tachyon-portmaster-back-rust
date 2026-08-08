//! Remover um produto.

use crate::context::UserContext;

/// Remover um produto.
#[derive(Debug, Clone)]
pub struct DeleteProductCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Id do produto, em base62.
    pub id: String,
}
