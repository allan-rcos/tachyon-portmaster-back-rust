//! O DTO de JSON de `CargoManifestItem`.

use serde::Serialize;

/// `CargoManifestItem` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct CargoManifestItemJson {
    /// Identidade do produto.
    pub(crate) product_id: String,
    /// Nome do produto.
    pub(crate) product_name: String,
    /// Quantas unidades estão a bordo.
    pub(crate) quantity: f64,
    /// Quanto elas pesam.
    pub(crate) weight: f64,
}
