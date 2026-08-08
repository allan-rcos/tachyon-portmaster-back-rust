//! A leitura de um `ProductViewItem` a partir de uma linha.

use portmaster_domain::enums::RiskClass;
use sqlx::mysql::MySqlRow;

use crate::query::row::Row;
use crate::query::views::ProductViewItem;

/// Lê um produto de uma linha de consulta.
pub(crate) struct ProductReader;

impl ProductReader {
    /// Uma linha de `products` como a View a quer.
    pub(crate) fn item(row: &MySqlRow) -> anyhow::Result<ProductViewItem> {
        Ok(ProductViewItem {
            id: Row::id(row, "id")?,
            name: Row::text(row, "name")?,
            density: Row::real(row, "density")?,
            risk_class: Row::enum_index(row, "risk_class", RiskClass::from_i32, "RiskClass")?,
        })
    }
}
