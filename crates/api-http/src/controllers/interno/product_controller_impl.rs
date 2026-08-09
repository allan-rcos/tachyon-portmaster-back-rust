//! O controller de produtos. Não sai do módulo.

use portmaster_app::commands::product::{
    CreateProductCommand, DeleteProductCommand, UpdateProductCommand,
};
use portmaster_app::context::UserContext;
use portmaster_app::domain::RiskClass;
use portmaster_app::queries::product::{GetProductQuery, ListProductsQuery};
use portmaster_app::services::ProductUseCase;

use crate::controllers::params::page_params::PageParams;
use crate::controllers::product_controller::ProductController;
use crate::error::api_error::ApiError;
use crate::wire::vo::common::risk_class_x::RiskClassX;
use crate::wire::vo::product::product_create_x_request::ProductCreateXRequest;
use crate::wire::vo::product::product_list_x_response::ProductListXResponse;
use crate::wire::vo::product::product_update_x_request::ProductUpdateXRequest;
use crate::wire::vo::product::product_x_response::ProductXResponse;

/// Os handlers de produto, genéricos sobre o caso de uso.
#[derive(Clone)]
pub(crate) struct ProductControllerImpl<U> {
    /// O caso de uso de produto.
    products: U,
}

impl<U: ProductUseCase> ProductControllerImpl<U> {
    /// Monta o controller.
    pub(crate) const fn new(products: U) -> Self {
        Self { products }
    }
}

impl<U: ProductUseCase + Clone + Send + Sync + 'static> ProductController
    for ProductControllerImpl<U>
{
    async fn list(
        &self,
        context: UserContext,
        params: PageParams,
    ) -> Result<ProductListXResponse, ApiError> {
        let view = self
            .products
            .list(ListProductsQuery {
                context,
                cursor: params.cursor,
                limit: params.limit,
                search: params.search,
            })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ProductListXResponse::of(view))
    }

    async fn create(
        &self,
        context: UserContext,
        request: ProductCreateXRequest,
    ) -> Result<ProductXResponse, ApiError> {
        let product = self
            .products
            .create(CreateProductCommand {
                context,
                name: request.name.unwrap_or_default(),
                density: request.density.unwrap_or_default(),
                risk_class: risk_class_of(request.risk_class),
            })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ProductXResponse::of_domain(product.as_ref()))
    }

    async fn get(&self, context: UserContext, id: String) -> Result<ProductXResponse, ApiError> {
        let view = self
            .products
            .get(GetProductQuery { context, id })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ProductXResponse::of(view))
    }

    async fn update(
        &self,
        context: UserContext,
        id: String,
        request: ProductUpdateXRequest,
    ) -> Result<ProductXResponse, ApiError> {
        let product = self
            .products
            .update(UpdateProductCommand {
                context,
                id,
                name: request.name.unwrap_or_default(),
                density: request.density.unwrap_or_default(),
                risk_class: risk_class_of(request.risk_class),
            })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ProductXResponse::of_domain(product.as_ref()))
    }

    async fn delete(&self, context: UserContext, id: String) -> Result<(), ApiError> {
        self.products
            .delete(DeleteProductCommand { context, id })
            .await
            .map_err(ApiError::of_app)
    }
}

/// A classe de risco do domínio a partir da que veio no fio.
///
/// Ausente vira `None` — a classe neutra, e não uma recusa: o `.fbs` publica
/// `None` como valor legítimo do enum, e um produto sem classificação é um caso
/// que o cadastro admite.
fn risk_class_of(value: Option<RiskClassX>) -> RiskClass {
    value
        .and_then(|class| RiskClass::from_i32(class.as_index()))
        .unwrap_or(RiskClass::None)
}
