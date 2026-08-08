//! Um registro de telemetria.

use crate::error::api_error::ApiError;
use crate::wire::convert::Convert;
use crate::wire::factory::response_factory::ResponseFactory;
use crate::wire::tables as fbs;
use portmaster_app::views::TelemetryLogView;

/// Monta a tabela de um registro de telemetria.
pub(crate) struct TelemetryLogItemFactory {
    source: TelemetryLogView,
}

impl TelemetryLogItemFactory {
    /// Monta a factory sobre a View.
    pub(crate) const fn of(source: TelemetryLogView) -> Self {
        Self { source }
    }
}

impl ResponseFactory for TelemetryLogItemFactory {
    type Table = fbs::container::TelemetryLogItem;

    fn table(&self) -> Result<Self::Table, ApiError> {
        Ok(fbs::container::TelemetryLogItem {
            id: Some(self.source.id.clone()),
            description: self.source.description.clone(),
            event: Convert::telemetry_event(self.source.event),
            timestamp: Convert::timestamp(self.source.timestamp),
        })
    }
}
