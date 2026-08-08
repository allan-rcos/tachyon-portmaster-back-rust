//! O read model `MetricsView`.

use crate::query::views::OccupancyView;
use serde::{Deserialize, Serialize};

/// O painel do pátio.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct MetricsView {
    /// Contêineres em qualquer status que não `Empty`.
    pub active_containers: i64,
    /// Contêineres registrados.
    pub total_containers: i64,
    /// Peso total embarcado no pátio.
    pub yard_load: f64,
    /// Produtos cadastrados.
    pub registered_products: i64,
    /// A distribuição por status.
    pub occupancy: OccupancyView,
}
