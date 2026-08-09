//! O DTO de JSON de `TelemetryLogItem`.

use crate::wire::dto::json::common::telemetry_event_json::TelemetryEventJson;
use serde::Serialize;

/// `TelemetryLogItem` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct TelemetryLogItemJson {
    /// Identidade da entrada.
    pub(crate) id: String,
    /// O que aconteceu.
    pub(crate) event: TelemetryEventJson,
    /// O que o operador anotou, quando anotou.
    pub(crate) description: Option<String>,
    /// Quando, em RFC 3339 e UTC.
    pub(crate) timestamp: Option<String>,
}
