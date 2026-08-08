//! O read model `TelemetryLogView`.

use serde::{Deserialize, Serialize};

/// Um registro de telemetria.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetryLogView {
    /// Id em base62.
    pub id: String,
    /// Índice de [`TelemetryEvent`](portmaster_domain::enums::TelemetryEvent).
    pub event: i32,
    /// Descrição livre, quando houver.
    pub description: Option<String>,
    /// Epoch em ms.
    pub timestamp: i64,
}
