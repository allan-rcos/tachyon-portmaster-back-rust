//! O read model `ContainerViewItem`.

use serde::{Deserialize, Serialize};

/// Um contêiner.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContainerViewItem {
    /// Id em base62.
    pub id: String,
    /// Código de identificação no pátio.
    pub code: String,
    /// Peso embarcado agora.
    pub current_weight: f64,
    /// Capacidade máxima.
    pub max_capacity: f64,
    /// Índice de [`ContainerStatus`](portmaster_domain::enums::ContainerStatus).
    pub status: i32,
}
