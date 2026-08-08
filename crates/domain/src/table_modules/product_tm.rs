//! As regras de produto.

use crate::enums::RiskClass;
use crate::error::ProductError;
use crate::models::Product;

/// Constrói e altera produtos.
pub trait ProductTM {
    /// Cria um produto novo, ainda não persistido.
    fn create(
        &self,
        name: String,
        density: f64,
        risk_class: RiskClass,
    ) -> Result<Box<dyn Product>, ProductError>;

    /// Produz o produto com outros dados de catálogo.
    fn update(
        &self,
        product: &dyn Product,
        name: String,
        density: f64,
        risk_class: RiskClass,
    ) -> Result<Box<dyn Product>, ProductError>;
}
