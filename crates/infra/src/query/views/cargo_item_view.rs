//! O read model `CargoItemView`.

use serde::{Deserialize, Serialize};

/// Uma linha do manifesto de carga.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CargoItemView {
    /// Id do produto, em base62.
    pub product_id: String,
    /// Nome do produto, para a listagem não exigir uma segunda consulta.
    pub product_name: String,
    /// Quantidade embarcada.
    pub quantity: f64,
    /// Peso correspondente.
    pub weight: f64,
}
