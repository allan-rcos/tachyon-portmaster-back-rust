//! Cadastrar um produto.

use crate::context::UserContext;
use portmaster_domain::enums::RiskClass;

/// Cadastrar um produto.
#[derive(Debug, Clone)]
pub struct CreateProductCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Nome do produto.
    pub name: String,
    /// Densidade, para converter quantidade em peso.
    pub density: f64,
    /// Classe de risco.
    pub risk_class: RiskClass,
}
