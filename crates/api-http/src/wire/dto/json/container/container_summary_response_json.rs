//! O DTO de JSON de `ContainerSummaryResponse`.

use crate::wire::dto::json::container::cargo_manifest_item_json::CargoManifestItemJson;
use crate::wire::dto::json::container::container_response_json::ContainerResponseJson;
use crate::wire::dto::json::container::telemetry_log_item_json::TelemetryLogItemJson;
use serde::Serialize;

/// `ContainerSummaryResponse` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct ContainerSummaryResponseJson {
    /// O contêiner em si.
    pub(crate) container: ContainerResponseJson,
    /// A carga a bordo.
    pub(crate) manifest: Vec<CargoManifestItemJson>,
    /// As últimas movimentações.
    pub(crate) recent_logs: Vec<TelemetryLogItemJson>,
}
