//! O VO de `MetricsResponse`.

use crate::wire::dto::json::metrics::metrics_response_json::MetricsResponseJson;
use crate::wire::tables as fbs;
use crate::wire::vo::metrics::occupancy_division_x::OccupancyDivisionX;
use crate::wire::x::response_x::ResponseX;
use portmaster_app::views::MetricsView;

/// O que a rota de `MetricsResponse` responde.
#[derive(Debug, Clone)]
pub(crate) struct MetricsXResponse {
    /// Contêineres em uso.
    pub(crate) active_containers: i32,
    /// Contêineres cadastrados.
    pub(crate) total_containers: i32,
    /// O peso total no pátio.
    pub(crate) yard_load: f64,
    /// Produtos cadastrados.
    pub(crate) registered_products: i32,
    /// A divisão por status.
    pub(crate) occupancy_division: OccupancyDivisionX,
}

impl ResponseX for MetricsXResponse {
    type Json = MetricsResponseJson;
    type Fbs = fbs::metrics::MetricsResponse;

    fn to_json(&self) -> Self::Json {
        MetricsResponseJson {
            active_containers: self.active_containers,
            total_containers: self.total_containers,
            yard_load: self.yard_load,
            registered_products: self.registered_products,
            occupancy_division: self.occupancy_division.to_json(),
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::metrics::MetricsResponse {
            active_containers: self.active_containers,
            total_containers: self.total_containers,
            yard_load: self.yard_load,
            registered_products: self.registered_products,
            occupancy_division: Some(Box::new(self.occupancy_division.to_fbs())),
        }
    }
}

impl MetricsXResponse {
    /// O painel do pátio.
    pub(crate) fn of(source: MetricsView) -> Self {
        Self {
            active_containers: i32::try_from(source.active_containers).unwrap_or(i32::MAX),
            total_containers: i32::try_from(source.total_containers).unwrap_or(i32::MAX),
            yard_load: source.yard_load,
            registered_products: i32::try_from(source.registered_products).unwrap_or(i32::MAX),
            occupancy_division: OccupancyDivisionX::of(source.occupancy),
        }
    }
}
