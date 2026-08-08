//! O read model `OccupancyView`.

use serde::{Deserialize, Serialize};

/// Quantos contêineres há em cada status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct OccupancyView {
    /// Registrados e sem carga.
    pub empty: i64,
    /// Recebendo carga.
    pub loading: i64,
    /// Fechados, aguardando despacho.
    pub sealed: i64,
    /// Despachados.
    pub in_transit: i64,
}
