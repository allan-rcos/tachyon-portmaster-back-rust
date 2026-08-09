//! O VO de `ContainerSummaryResponse`.

use crate::wire::dto::json::container::container_summary_response_json::ContainerSummaryResponseJson;
use crate::wire::tables as fbs;
use crate::wire::vo::container::cargo_manifest_item_x::CargoManifestItemX;
use crate::wire::vo::container::container_x_response::ContainerXResponse;
use crate::wire::vo::container::telemetry_log_item_x::TelemetryLogItemX;
use crate::wire::x::response_x::ResponseX;
use portmaster_app::views::ContainerSummaryViewItem;

/// O que a rota de `ContainerSummaryResponse` responde.
#[derive(Debug, Clone)]
pub(crate) struct ContainerSummaryXResponse {
    /// O contêiner em si.
    pub(crate) container: ContainerXResponse,
    /// A carga a bordo.
    pub(crate) manifest: Vec<CargoManifestItemX>,
    /// As últimas movimentações.
    pub(crate) recent_logs: Vec<TelemetryLogItemX>,
}

impl ResponseX for ContainerSummaryXResponse {
    type Json = ContainerSummaryResponseJson;
    type Fbs = fbs::container::ContainerSummaryResponse;

    fn to_json(&self) -> Self::Json {
        ContainerSummaryResponseJson {
            container: self.container.to_json(),
            manifest: self.manifest.iter().map(ResponseX::to_json).collect(),
            recent_logs: self.recent_logs.iter().map(ResponseX::to_json).collect(),
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::container::ContainerSummaryResponse {
            container: Some(Box::new(self.container.to_fbs())),
            manifest: Some(self.manifest.iter().map(ResponseX::to_fbs).collect()),
            recent_logs: Some(self.recent_logs.iter().map(ResponseX::to_fbs).collect()),
        }
    }
}

impl ContainerSummaryXResponse {
    /// O resumo de um contêiner.
    pub(crate) fn of(source: ContainerSummaryViewItem) -> Self {
        Self {
            container: ContainerXResponse::of(source.container),
            manifest: source.manifest.into_iter().map(CargoManifestItemX::of).collect(),
            recent_logs: source.recent_logs.into_iter().map(TelemetryLogItemX::of).collect(),
        }
    }
}
