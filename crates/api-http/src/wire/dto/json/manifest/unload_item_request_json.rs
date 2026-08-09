//! O DTO de JSON de `UnloadItemRequest`.

use serde::Deserialize;

/// `UnloadItemRequest` como o serde o lê.
///
/// Todo campo é opcional: um que falte chega como `None` e o `TableModule`
/// o recusa nomeando-o, em lote com os demais.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct UnloadItemRequestJson {
    /// De qual contêiner desembarcar.
    pub(crate) container_id: Option<String>,
    /// O que desembarcar.
    pub(crate) product_id: Option<String>,
    /// Quantas unidades.
    pub(crate) quantity: Option<f64>,
}
