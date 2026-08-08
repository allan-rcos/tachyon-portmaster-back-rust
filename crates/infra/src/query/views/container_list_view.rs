//! O read model `ContainerListView`.

use crate::query::views::ContainerViewItem;
use serde::{Deserialize, Serialize};

/// A listagem de contêineres.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContainerListView {
    /// Os contêineres da página.
    pub items: Vec<ContainerViewItem>,
    /// Token da próxima página, ou `None` se esta foi a última.
    pub next_cursor: Option<String>,
    /// Quantos contêineres o filtro alcança ao todo.
    pub total: i64,
}
