//! O read model `RoleListView`.

use crate::query::views::RoleViewItem;
use serde::{Deserialize, Serialize};

/// A listagem de papéis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleListView {
    /// Os papéis da página.
    pub items: Vec<RoleViewItem>,
    /// Token da próxima página, ou `None` se esta foi a última.
    pub next_cursor: Option<String>,
    /// Quantos papéis o filtro alcança ao todo.
    pub total: i64,
}
