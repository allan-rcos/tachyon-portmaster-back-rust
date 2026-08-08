//! O painel do pátio.

use crate::error::api_error::ApiError;
use crate::wire::convert::Convert;
use crate::wire::dto::metrics::occupancy_division_factory::OccupancyDivisionFactory;
use crate::wire::factory::response_factory::ResponseFactory;
use crate::wire::tables as fbs;
use portmaster_app::views::MetricsView;

/// Monta a tabela do painel.
pub(crate) struct MetricsResponseFactory {
    /// A View de origem, que a `table()` traduz.
    source: MetricsView,
}

impl MetricsResponseFactory {
    /// Monta a factory sobre a View.
    pub(crate) const fn of(source: MetricsView) -> Self {
        Self { source }
    }
}

impl ResponseFactory for MetricsResponseFactory {
    type Table = fbs::metrics::MetricsResponse;

    fn table(&self) -> Result<Self::Table, ApiError> {
        Ok(fbs::metrics::MetricsResponse {
            active_containers: Convert::count(self.source.active_containers),
            total_containers: Convert::count(self.source.total_containers),
            yard_load: self.source.yard_load,
            registered_products: Convert::count(self.source.registered_products),
            occupancy_division: Some(Box::new(
                OccupancyDivisionFactory::of(self.source.occupancy).table()?,
            )),
        })
    }
}
