//! Um contêiner com a carga e a telemetria recente.

use crate::error::api_error::ApiError;
use crate::wire::dto::container::cargo_manifest_item_factory::CargoManifestItemFactory;
use crate::wire::dto::container::container_response_factory::ContainerResponseFactory;
use crate::wire::dto::container::telemetry_log_item_factory::TelemetryLogItemFactory;
use crate::wire::factory::response_factory::ResponseFactory;
use crate::wire::tables as fbs;
use portmaster_app::views::ContainerSummaryViewItem;

/// Monta a tabela do resumo de um contêiner.
pub(crate) struct ContainerSummaryResponseFactory {
    /// A View de origem, que a `table()` traduz.
    source: ContainerSummaryViewItem,
}

impl ContainerSummaryResponseFactory {
    /// Monta a factory sobre a View.
    pub(crate) const fn of(source: ContainerSummaryViewItem) -> Self {
        Self { source }
    }
}

impl ResponseFactory for ContainerSummaryResponseFactory {
    type Table = fbs::container::ContainerSummaryResponse;

    fn table(&self) -> Result<Self::Table, ApiError> {
        Ok(fbs::container::ContainerSummaryResponse {
            container: Some(Box::new(
                ContainerResponseFactory::of(self.source.container.clone()).table()?,
            )),
            manifest: Some(
                self.source
                    .manifest
                    .iter()
                    .map(|item| CargoManifestItemFactory::of(item.clone()).table())
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            recent_logs: Some(
                self.source
                    .recent_logs
                    .iter()
                    .map(|item| TelemetryLogItemFactory::of(item.clone()).table())
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        })
    }
}
