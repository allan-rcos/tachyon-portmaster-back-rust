//! O DTO de JSON de `ProductListResponse`.

use crate::wire::dto::json::product::product_response_json::ProductResponseJson;
use serde::Serialize;

/// `ProductListResponse` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct ProductListResponseJson {
    /// A página de produtos.
    pub(crate) data: Vec<ProductResponseJson>,
    /// Por onde continuar, ou `None` na última página.
    pub(crate) next_cursor: Option<String>,
    /// Quantos produtos existem ao todo.
    pub(crate) total: i32,
}
