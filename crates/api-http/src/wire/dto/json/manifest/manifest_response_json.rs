//! O DTO de JSON de `ManifestResponse`.

use crate::wire::dto::json::container::container_response_json::ContainerResponseJson;
use serde::Serialize;

/// `ManifestResponse` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct ManifestResponseJson {
    /// O que aconteceu, em texto.
    pub(crate) message: String,
    /// O contêiner depois da movimentação.
    pub(crate) container: ContainerResponseJson,
}
