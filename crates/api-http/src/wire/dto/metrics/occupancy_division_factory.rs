//! A divisão de ocupação do pátio.

use crate::error::api_error::ApiError;
use crate::wire::convert::Convert;
use crate::wire::factory::response_factory::ResponseFactory;
use crate::wire::tables as fbs;
use portmaster_app::views::OccupancyView;

/// Monta a tabela da ocupação.
pub(crate) struct OccupancyDivisionFactory {
    /// A View de origem, que a `table()` traduz.
    source: OccupancyView,
}

impl OccupancyDivisionFactory {
    /// Monta a factory sobre a View.
    pub(crate) const fn of(source: OccupancyView) -> Self {
        Self { source }
    }
}

impl ResponseFactory for OccupancyDivisionFactory {
    type Table = fbs::metrics::OccupancyDivision;

    fn table(&self) -> Result<Self::Table, ApiError> {
        Ok(fbs::metrics::OccupancyDivision {
            empty: Convert::count(self.source.empty),
            loading: Convert::count(self.source.loading),
            sealed: Convert::count(self.source.sealed),
            in_transit: Convert::count(self.source.in_transit),
        })
    }
}
