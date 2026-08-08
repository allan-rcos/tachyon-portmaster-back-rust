//! Listar produtos.

use crate::context::UserContext;

/// Listar produtos.
#[derive(Debug, Clone)]
pub struct ListProductsQuery {
    /// Quem está consultando.
    pub context: UserContext,
    /// Token da página anterior.
    pub cursor: Option<String>,
    /// Tamanho da página.
    pub limit: Option<u32>,
    /// Termo de busca.
    pub search: Option<String>,
}
