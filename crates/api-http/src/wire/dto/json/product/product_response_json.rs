//! O DTO de JSON de `ProductResponse`.

use crate::wire::dto::json::common::risk_class_json::RiskClassJson;
use serde::Serialize;

/// `ProductResponse` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct ProductResponseJson {
    /// Identidade, em base62.
    pub(crate) id: String,
    /// Nome do produto.
    pub(crate) name: String,
    /// A densidade.
    pub(crate) density: f64,
    /// A classe de risco.
    pub(crate) risk_class: RiskClassJson,
}
