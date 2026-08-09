//! O DTO de JSON de `ProductCreateRequest`.

use crate::wire::dto::json::common::risk_class_json::RiskClassJson;
use serde::Deserialize;

/// `ProductCreateRequest` como o serde o lê.
///
/// Todo campo é opcional: um que falte chega como `None` e o `TableModule`
/// o recusa nomeando-o, em lote com os demais.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct ProductCreateRequestJson {
    /// O nome do produto novo.
    pub(crate) name: Option<String>,
    /// A densidade, que converte quantidade em peso.
    pub(crate) density: Option<f64>,
    /// A classe de risco.
    pub(crate) risk_class: Option<RiskClassJson>,
}
