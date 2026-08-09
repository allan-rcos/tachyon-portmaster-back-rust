//! O DTO de JSON de `OccupancyDivision`.

use serde::Serialize;

/// `OccupancyDivision` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct OccupancyDivisionJson {
    /// Quantos contêineres estão vazios.
    pub(crate) empty: i32,
    /// Quantos estão carregando.
    pub(crate) loading: i32,
    /// Quantos estão selados.
    pub(crate) sealed: i32,
    /// Quantos estão em trânsito.
    pub(crate) in_transit: i32,
}
