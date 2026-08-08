//! O read model `ProductListView`.

use crate::query::views::ProductViewItem;
use serde::{Deserialize, Serialize};

/// A listagem de produtos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductListView {
    /// Os produtos da página.
    pub items: Vec<ProductViewItem>,
    /// Token da próxima página, ou `None` se esta foi a última.
    pub next_cursor: Option<String>,
    /// Quantos produtos o filtro alcança ao todo.
    pub total: i64,
}
