//! O DTO de JSON de `LoadItemRequest`.

use serde::Deserialize;

/// `LoadItemRequest` como o serde o lê.
///
/// Todo campo é opcional: um que falte chega como `None` e o `TableModule`
/// o recusa nomeando-o, em lote com os demais.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct LoadItemRequestJson {
    /// Em qual contêiner embarcar.
    pub(crate) container_id: Option<String>,
    /// O que embarcar.
    pub(crate) product_id: Option<String>,
    /// Quantas unidades.
    pub(crate) quantity: Option<f64>,
}
