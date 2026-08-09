//! O DTO de JSON de `ProductUpdateRequest`.

use crate::wire::dto::json::common::risk_class_json::RiskClassJson;
use serde::Deserialize;

/// `ProductUpdateRequest` como o serde o lê.
///
/// Todo campo é opcional: um que falte chega como `None` e o `TableModule`
/// o recusa nomeando-o, em lote com os demais.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct ProductUpdateRequestJson {
    /// O nome novo.
    pub(crate) name: Option<String>,
    /// A densidade nova.
    pub(crate) density: Option<f64>,
    /// A classe de risco nova.
    pub(crate) risk_class: Option<RiskClassJson>,
}
