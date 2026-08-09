//! O DTO de JSON de `ContainerUpdateRequest`.

use serde::Deserialize;

/// `ContainerUpdateRequest` como o serde o lê.
///
/// Todo campo é opcional: um que falte chega como `None` e o `TableModule`
/// o recusa nomeando-o, em lote com os demais.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct ContainerUpdateRequestJson {
    /// A capacidade máxima nova.
    pub(crate) max_capacity: Option<f64>,
}
