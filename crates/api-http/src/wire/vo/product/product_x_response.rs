//! O VO de `ProductResponse`.

use crate::wire::dto::json::product::product_response_json::ProductResponseJson;
use crate::wire::tables as fbs;
use crate::wire::vo::common::risk_class_x::RiskClassX;
use crate::wire::x::response_x::ResponseX;
use portmaster_app::views::ProductViewItem;

/// O que a rota de `ProductResponse` responde.
#[derive(Debug, Clone)]
pub(crate) struct ProductXResponse {
    /// Identidade, em base62.
    pub(crate) id: String,
    /// Nome do produto.
    pub(crate) name: String,
    /// A densidade.
    pub(crate) density: f64,
    /// A classe de risco.
    pub(crate) risk_class: RiskClassX,
}

impl ResponseX for ProductXResponse {
    type Json = ProductResponseJson;
    type Fbs = fbs::product::ProductResponse;

    fn to_json(&self) -> Self::Json {
        ProductResponseJson {
            id: self.id.clone(),
            name: self.name.clone(),
            density: self.density,
            risk_class: self.risk_class.to_json(),
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::product::ProductResponse {
            id: Some(self.id.clone()),
            name: Some(self.name.clone()),
            density: self.density,
            risk_class: self.risk_class.to_fbs(),
        }
    }
}

impl ProductXResponse {
    /// O produto, vindo do lado de leitura.
    pub(crate) fn of(source: ProductViewItem) -> Self {
        Self {
            id: source.id,
            name: source.name,
            density: source.density,
            risk_class: RiskClassX::of_index(source.risk_class),
        }
    }

    /// O produto, vindo do objeto de domínio.
    pub(crate) fn of_domain(product: &dyn portmaster_app::domain::Product) -> Self {
        Self {
            id: product.id().to_owned(),
            name: product.name().to_owned(),
            density: product.density(),
            risk_class: RiskClassX::of_index(product.risk_class().as_i32()),
        }
    }
}
