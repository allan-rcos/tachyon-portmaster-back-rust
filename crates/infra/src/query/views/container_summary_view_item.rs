//! O read model `ContainerSummaryViewItem`.

use serde::{Deserialize, Serialize};

use crate::query::views::{CargoItemView, ContainerViewItem, TelemetryLogView};

/// Um contêiner com a carga e o histórico recente.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContainerSummaryViewItem {
    /// O contêiner em si.
    pub container: ContainerViewItem,
    /// A carga embarcada agora.
    pub manifest: Vec<CargoItemView>,
    /// Os últimos registros de telemetria, do mais antigo ao mais novo.
    pub recent_logs: Vec<TelemetryLogView>,
}
