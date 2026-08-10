//! O VO de `TelemetryLogItem`.

use crate::wire::convert::Convert;
use crate::wire::dto::json::container::telemetry_log_item_json::TelemetryLogItemJson;
use crate::wire::tables as fbs;
use crate::wire::vo::common::telemetry_event_x::TelemetryEventX;
use crate::wire::x::response_x::ResponseX;
use portmaster_app::views::TelemetryLogView;

/// O que a rota de `TelemetryLogItem` responde.
#[derive(Debug, Clone)]
pub(crate) struct TelemetryLogItemX {
    /// Identidade da entrada.
    pub(crate) id: String,
    /// O que aconteceu.
    pub(crate) event: TelemetryEventX,
    /// O que o operador anotou, quando anotou.
    pub(crate) description: Option<String>,
    /// Quando, em RFC 3339 e UTC.
    pub(crate) timestamp: Option<String>,
}

impl ResponseX for TelemetryLogItemX {
    type Json = TelemetryLogItemJson;
    type Fbs = fbs::container::TelemetryLogItem;

    fn to_json(&self) -> Self::Json {
        TelemetryLogItemJson {
            id: self.id.clone(),
            event: self.event.to_json(),
            description: self.description.clone(),
            timestamp: self.timestamp.clone(),
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::container::TelemetryLogItem {
            id: Some(self.id.clone()),
            event: self.event.to_fbs(),
            description: self.description.clone(),
            timestamp: self.timestamp.clone(),
        }
    }
}

impl TelemetryLogItemX {
    /// Uma entrada de telemetria.
    pub(crate) fn of(source: TelemetryLogView) -> Self {
        Self {
            id: source.id,
            event: TelemetryEventX::of_index(source.event),
            description: source.description,
            timestamp: Convert::timestamp(source.timestamp),
        }
    }
}
