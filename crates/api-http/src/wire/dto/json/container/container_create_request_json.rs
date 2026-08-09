//! O DTO de JSON de `ContainerCreateRequest`.

use serde::Deserialize;

/// `ContainerCreateRequest` como o serde o lê.
///
/// Todo campo é opcional: um que falte chega como `None` e o `TableModule`
/// o recusa nomeando-o, em lote com os demais.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct ContainerCreateRequestJson {
    /// O código do contêiner.
    pub(crate) code: Option<String>,
    /// A capacidade máxima, em quilos.
    pub(crate) max_capacity: Option<f64>,
}
