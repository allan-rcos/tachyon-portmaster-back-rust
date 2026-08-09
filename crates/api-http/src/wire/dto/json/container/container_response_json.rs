//! O DTO de JSON de `ContainerResponse`.

use crate::wire::dto::json::common::container_status_json::ContainerStatusJson;
use serde::Serialize;

/// `ContainerResponse` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct ContainerResponseJson {
    /// Identidade, em base62.
    pub(crate) id: String,
    /// O código do contêiner.
    pub(crate) code: String,
    /// O peso já embarcado.
    pub(crate) current_weight: f64,
    /// A capacidade máxima.
    pub(crate) max_capacity: f64,
    /// Em que ponto do ciclo ele está.
    pub(crate) status: ContainerStatusJson,
}
