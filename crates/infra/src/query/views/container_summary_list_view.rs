//! O read model `ContainerSummaryListView`.

use crate::query::views::ContainerSummaryViewItem;
use serde::{Deserialize, Serialize};

/// A listagem de resumos de contêiner.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContainerSummaryListView {
    /// Os resumos da página.
    pub items: Vec<ContainerSummaryViewItem>,
    /// Token da próxima página, ou `None` se esta foi a última.
    pub next_cursor: Option<String>,
    /// Quantos contêineres o filtro alcança ao todo.
    pub total: i64,
}
