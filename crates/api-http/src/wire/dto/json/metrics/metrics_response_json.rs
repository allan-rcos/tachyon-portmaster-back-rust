//! O DTO de JSON de `MetricsResponse`.

use crate::wire::dto::json::metrics::occupancy_division_json::OccupancyDivisionJson;
use serde::Serialize;

/// `MetricsResponse` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct MetricsResponseJson {
    /// Contêineres em uso.
    pub(crate) active_containers: i32,
    /// Contêineres cadastrados.
    pub(crate) total_containers: i32,
    /// O peso total no pátio.
    pub(crate) yard_load: f64,
    /// Produtos cadastrados.
    pub(crate) registered_products: i32,
    /// A divisão por status.
    pub(crate) occupancy_division: OccupancyDivisionJson,
}
