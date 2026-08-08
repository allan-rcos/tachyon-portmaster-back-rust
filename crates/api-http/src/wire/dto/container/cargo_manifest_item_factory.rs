//! Uma linha do manifesto.

use crate::error::api_error::ApiError;
use crate::wire::factory::response_factory::ResponseFactory;
use crate::wire::tables as fbs;
use portmaster_app::views::CargoItemView;

/// Monta a tabela de uma linha de carga.
pub(crate) struct CargoManifestItemFactory {
    source: CargoItemView,
}

impl CargoManifestItemFactory {
    /// Monta a factory sobre a View.
    pub(crate) const fn of(source: CargoItemView) -> Self {
        Self { source }
    }
}

impl ResponseFactory for CargoManifestItemFactory {
    type Table = fbs::container::CargoManifestItem;

    fn table(&self) -> Result<Self::Table, ApiError> {
        Ok(fbs::container::CargoManifestItem {
            product_id: Some(self.source.product_id.clone()),
            product_name: Some(self.source.product_name.clone()),
            quantity: self.source.quantity,
            weight: self.source.weight,
        })
    }
}
