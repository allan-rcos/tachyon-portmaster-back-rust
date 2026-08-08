//! Alterar um produto.

use crate::context::UserContext;
use portmaster_domain::enums::RiskClass;

/// Alterar um produto.
#[derive(Debug, Clone)]
pub struct UpdateProductCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Id do produto, em base62.
    pub id: String,
    /// Nome do produto.
    pub name: String,
    /// Densidade.
    pub density: f64,
    /// Classe de risco.
    pub risk_class: RiskClass,
}
