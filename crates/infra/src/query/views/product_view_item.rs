//! O read model `ProductViewItem`.

use serde::{Deserialize, Serialize};

/// Um produto.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductViewItem {
    /// Id em base62.
    pub id: String,
    /// Nome do produto.
    pub name: String,
    /// Densidade, para converter quantidade em peso.
    pub density: f64,
    /// Índice de [`RiskClass`](portmaster_domain::enums::RiskClass).
    pub risk_class: i32,
}
