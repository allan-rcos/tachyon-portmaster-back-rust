//! O contrato do controller de produtos.

use portmaster_app::context::UserContext;

use crate::controllers::params::page_params::PageParams;
use crate::error::api_error::ApiError;
use crate::wire::vo::product::product_create_x_request::ProductCreateXRequest;
use crate::wire::vo::product::product_list_x_response::ProductListXResponse;
use crate::wire::vo::product::product_update_x_request::ProductUpdateXRequest;
use crate::wire::vo::product::product_x_response::ProductXResponse;

/// Os handlers de produto.
#[trait_variant::make(Send)]
pub(crate) trait ProductController: Clone + Sync + 'static {
    /// `GET /products`
    async fn list(
        &self,
        context: UserContext,
        params: PageParams,
    ) -> Result<ProductListXResponse, ApiError>;

    /// `POST /products`
    async fn create(
        &self,
        context: UserContext,
        request: ProductCreateXRequest,
    ) -> Result<ProductXResponse, ApiError>;

    /// `GET /products/{id}`
    async fn get(&self, context: UserContext, id: String) -> Result<ProductXResponse, ApiError>;

    /// `PUT /products/{id}`
    async fn update(
        &self,
        context: UserContext,
        id: String,
        request: ProductUpdateXRequest,
    ) -> Result<ProductXResponse, ApiError>;

    /// `DELETE /products/{id}`
    async fn delete(&self, context: UserContext, id: String) -> Result<(), ApiError>;
}
