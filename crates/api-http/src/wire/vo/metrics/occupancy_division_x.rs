//! O VO de `OccupancyDivision`.

use crate::wire::dto::json::metrics::occupancy_division_json::OccupancyDivisionJson;
use crate::wire::tables as fbs;
use crate::wire::x::response_x::ResponseX;
use portmaster_app::views::OccupancyView;

/// O que a rota de `OccupancyDivision` responde.
#[derive(Debug, Clone)]
pub(crate) struct OccupancyDivisionX {
    /// Quantos contêineres estão vazios.
    pub(crate) empty: i32,
    /// Quantos estão carregando.
    pub(crate) loading: i32,
    /// Quantos estão selados.
    pub(crate) sealed: i32,
    /// Quantos estão em trânsito.
    pub(crate) in_transit: i32,
}

impl ResponseX for OccupancyDivisionX {
    type Json = OccupancyDivisionJson;
    type Fbs = fbs::metrics::OccupancyDivision;

    fn to_json(&self) -> Self::Json {
        OccupancyDivisionJson {
            empty: self.empty,
            loading: self.loading,
            sealed: self.sealed,
            in_transit: self.in_transit,
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::metrics::OccupancyDivision {
            empty: self.empty,
            loading: self.loading,
            sealed: self.sealed,
            in_transit: self.in_transit,
        }
    }
}

impl OccupancyDivisionX {
    /// A divisão por status.
    pub(crate) fn of(source: OccupancyView) -> Self {
        Self {
            empty: i32::try_from(source.empty).unwrap_or(i32::MAX),
            loading: i32::try_from(source.loading).unwrap_or(i32::MAX),
            sealed: i32::try_from(source.sealed).unwrap_or(i32::MAX),
            in_transit: i32::try_from(source.in_transit).unwrap_or(i32::MAX),
        }
    }
}
