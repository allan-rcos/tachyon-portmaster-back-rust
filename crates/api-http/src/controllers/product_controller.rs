//! O contrato do controller de produtos.

use crate::controllers::params::page_params::PageParams;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::vo::product::product_create_x_request::ProductCreateXRequest;
use crate::wire::vo::product::product_list_x_response::ProductListXResponse;
use crate::wire::vo::product::product_update_x_request::ProductUpdateXRequest;
use crate::wire::vo::product::product_x_response::ProductXResponse;
use axum::extract::{Path, Query};

/// Os handlers de produto.
#[trait_variant::make(Send)]
pub(crate) trait ProductController: Clone + Sync + 'static {
    /// `GET /products`
    async fn list(self, params: Query<PageParams>) -> ApiResponse<ProductListXResponse>;

    /// `POST /products`
    async fn create(self, request: Body<ProductCreateXRequest>) -> ApiResponse<ProductXResponse>;

    /// `GET /products/{id}`
    async fn get(self, id: Path<String>) -> ApiResponse<ProductXResponse>;

    /// `PUT /products/{id}`
    async fn update(
        self,
        id: Path<String>,
        request: Body<ProductUpdateXRequest>,
    ) -> ApiResponse<ProductXResponse>;

    /// `DELETE /products/{id}`
    async fn delete(self, id: Path<String>) -> ApiResponse;
}
