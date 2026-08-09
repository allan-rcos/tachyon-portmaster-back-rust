//! O VO de `CargoManifestItem`.

use crate::wire::dto::json::container::cargo_manifest_item_json::CargoManifestItemJson;
use crate::wire::tables as fbs;
use crate::wire::x::response_x::ResponseX;
use portmaster_app::views::CargoItemView;

/// O que a rota de `CargoManifestItem` responde.
#[derive(Debug, Clone)]
pub(crate) struct CargoManifestItemX {
    /// Identidade do produto.
    pub(crate) product_id: String,
    /// Nome do produto.
    pub(crate) product_name: String,
    /// Quantas unidades estão a bordo.
    pub(crate) quantity: f64,
    /// Quanto elas pesam.
    pub(crate) weight: f64,
}

impl ResponseX for CargoManifestItemX {
    type Json = CargoManifestItemJson;
    type Fbs = fbs::container::CargoManifestItem;

    fn to_json(&self) -> Self::Json {
        CargoManifestItemJson {
            product_id: self.product_id.clone(),
            product_name: self.product_name.clone(),
            quantity: self.quantity,
            weight: self.weight,
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::container::CargoManifestItem {
            product_id: Some(self.product_id.clone()),
            product_name: Some(self.product_name.clone()),
            quantity: self.quantity,
            weight: self.weight,
        }
    }
}

impl CargoManifestItemX {
    /// Um item da carga a bordo.
    pub(crate) fn of(source: CargoItemView) -> Self {
        Self {
            product_id: source.product_id,
            product_name: source.product_name,
            quantity: source.quantity,
            weight: source.weight,
        }
    }
}
