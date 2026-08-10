//! O VO de `ProductListResponse`.

use crate::wire::convert::Convert;
use crate::wire::dto::json::product::product_list_response_json::ProductListResponseJson;
use crate::wire::tables as fbs;
use crate::wire::vo::product::product_x_response::ProductXResponse;
use crate::wire::x::response_x::ResponseX;
use portmaster_app::views::ProductListView;

/// O que a rota de `ProductListResponse` responde.
#[derive(Debug, Clone)]
pub(crate) struct ProductListXResponse {
    /// A página de produtos.
    pub(crate) data: Vec<ProductXResponse>,
    /// Por onde continuar, ou `None` na última página.
    pub(crate) next_cursor: Option<String>,
    /// Quantos produtos existem ao todo.
    pub(crate) total: i32,
}

impl ResponseX for ProductListXResponse {
    type Json = ProductListResponseJson;
    type Fbs = fbs::product::ProductListResponse;

    fn to_json(&self) -> Self::Json {
        ProductListResponseJson {
            data: self.data.iter().map(ResponseX::to_json).collect(),
            next_cursor: self.next_cursor.clone(),
            total: self.total,
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::product::ProductListResponse {
            data: Some(self.data.iter().map(ResponseX::to_fbs).collect()),
            next_cursor: self.next_cursor.clone(),
            total: self.total,
        }
    }
}

impl ProductListXResponse {
    /// A página de produtos.
    pub(crate) fn of(source: ProductListView) -> Self {
        Self {
            data: source.items.into_iter().map(ProductXResponse::of).collect(),
            next_cursor: source.next_cursor,
            total: Convert::count(source.total),
        }
    }
}
