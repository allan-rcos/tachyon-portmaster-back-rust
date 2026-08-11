//! O controller de produtos. Não sai do módulo.

use axum::http::StatusCode;
use portmaster_app::commands::product::{
    CreateProductCommand, DeleteProductCommand, UpdateProductCommand,
};
use portmaster_app::domain::RiskClass;
use portmaster_app::error::ProductError;
use portmaster_app::queries::product::{GetProductQuery, ListProductsQuery};
use portmaster_app::services::ProductUseCase;

use crate::controllers::params::page_params::PageParams;
use crate::controllers::product_controller::ProductController;
use crate::middleware::session_port::SessionPort;
use crate::ports::error::api_error::ApiError;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::vo::common::risk_class_x::RiskClassX;
use crate::wire::vo::product::product_create_x_request::ProductCreateXRequest;
use crate::wire::vo::product::product_list_x_response::ProductListXResponse;
use crate::wire::vo::product::product_update_x_request::ProductUpdateXRequest;
use crate::wire::vo::product::product_x_response::ProductXResponse;
use axum::extract::{Path, Query};

/// Os handlers de produto, genéricos sobre o caso de uso.
#[derive(Clone)]
pub(crate) struct ProductControllerImpl<U, S> {
    /// O caso de uso de produto.
    products: U,
    /// Quem diz se há sessão, e quem a apresenta.
    session: S,
}

impl<U: ProductUseCase, S: SessionPort> ProductControllerImpl<U, S> {
    /// Monta o controller.
    pub(crate) const fn new(products: U, session: S) -> Self {
        Self { products, session }
    }
}

impl<U: ProductUseCase + Clone + Send + Sync + 'static, S: SessionPort> ProductController
    for ProductControllerImpl<U, S>
{
    async fn list(self, Query(params): Query<PageParams>) -> ApiResponse<ProductListXResponse> {
        ApiResponse::ok(
            async {
                let context = self.session.require_user()?;

                let view = self
                    .products
                    .list(ListProductsQuery {
                        context,
                        cursor: params.cursor,
                        limit: params.limit,
                        search: params.search,
                    })
                    .await
                    .map_err(to_api)?;

                Ok(ProductListXResponse::of(view))
            }
            .await,
        )
    }

    async fn create(
        self,
        Body(request): Body<ProductCreateXRequest>,
    ) -> ApiResponse<ProductXResponse> {
        ApiResponse::created(
            async {
                let context = self.session.require_user()?;

                let product = self
                    .products
                    .create(CreateProductCommand {
                        context,
                        name: request.name.unwrap_or_default(),
                        density: request.density.unwrap_or_default(),
                        risk_class: risk_class_of(request.risk_class),
                    })
                    .await
                    .map_err(to_api)?;

                Ok(ProductXResponse::of_domain(product.as_ref()))
            }
            .await,
        )
    }

    async fn get(self, Path(id): Path<String>) -> ApiResponse<ProductXResponse> {
        ApiResponse::ok(
            async {
                let context = self.session.require_user()?;

                let view = self
                    .products
                    .get(GetProductQuery { context, id })
                    .await
                    .map_err(to_api)?;

                Ok(ProductXResponse::of(view))
            }
            .await,
        )
    }

    async fn update(
        self,
        Path(id): Path<String>,
        Body(request): Body<ProductUpdateXRequest>,
    ) -> ApiResponse<ProductXResponse> {
        ApiResponse::ok(
            async {
                let context = self.session.require_user()?;

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
                    .map_err(to_api)?;

                Ok(ProductXResponse::of_domain(product.as_ref()))
            }
            .await,
        )
    }

    async fn delete(self, Path(id): Path<String>) -> ApiResponse {
        ApiResponse::no_content(
            async {
                let context = self.session.require_user()?;

                self.products
                    .delete(DeleteProductCommand { context, id })
                    .await
                    .map_err(to_api)
            }
            .await,
        )
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

/// Traduz a recusa do serviço de produtos no status que o cliente recebe.
fn to_api(error: ProductError) -> ApiError {
    match error {
        ProductError::Missing(id) => ApiError::new(
            StatusCode::NOT_FOUND,
            format!("Product {id} was not found."),
        ),
        ProductError::App(shared) => ApiError::of_app(shared),
    }
}
