//! `/products` — o molde dos demais recursos.
//!
//! Mostra os dois formatos de handler que se repetem no resto: os de **escrita**
//! (corpo → Command → objeto de domínio → tabela) e os de **leitura**
//! (querystring → Query → View → tabela).

use axum::extract::{Path, Query};
use portmaster_app::commands::product::CreateProductCommand;
use portmaster_app::commands::product::DeleteProductCommand;
use portmaster_app::commands::product::UpdateProductCommand;
use portmaster_app::queries::product::GetProductQuery;
use portmaster_app::queries::product::ListProductsQuery;
use portmaster_app::services::ProductUseCase;

use crate::error::api_error::ApiError;
use crate::handlers::params::page_params::PageParams;
use crate::session::Session;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::dto::product::product_create_request_factory::ProductCreateRequestFactory;
use crate::wire::dto::product::product_list_response_factory::ProductListResponseFactory;
use crate::wire::dto::product::product_response_factory::ProductResponseFactory;
use crate::wire::dto::product::product_update_request_factory::ProductUpdateRequestFactory;
use crate::wire::no_content::NoContent;
use crate::wire::wire::Wire;

/// Os handlers de produto, sobre o caso de uso que o provider entregou.
pub struct ProductHandlers<U> {
    products: U,
}

impl<U: ProductUseCase> ProductHandlers<U> {
    /// Monta os handlers.
    pub(crate) const fn new(products: U) -> Self {
        Self { products }
    }

    /// `GET /products`
    pub(crate) async fn list(
        &self,
        wire: Wire,
        Query(params): Query<PageParams>,
    ) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

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

        Ok(ApiResponse::ok(wire, ProductListResponseFactory::of(view)))
    }

    /// `POST /products`
    pub(crate) async fn create(
        &self,
        wire: Wire,
        Body(request): Body<ProductCreateRequestFactory>,
    ) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

        let product = self
            .products
            .create(CreateProductCommand {
                context,
                name: request.name.unwrap_or_default(),
                density: request.density.unwrap_or_default(),
                // O enum do wire e o do domínio têm os mesmos índices — os dois
                // saem do mesmo `.fbs`. A conversão passa pelo índice em vez de
                // casar variante a variante, para que acrescentar uma classe de
                // risco não exija tocar aqui.
                risk_class: risk_class_of(request.risk_class),
            })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ApiResponse::created(
            wire,
            ProductResponseFactory::of_domain(product.as_ref()),
        ))
    }

    /// `GET /products/{id}`
    pub(crate) async fn get(
        &self,
        wire: Wire,
        Path(id): Path<String>,
    ) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

        let view = self
            .products
            .get(GetProductQuery { context, id })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ApiResponse::ok(wire, ProductResponseFactory::of(view)))
    }

    /// `PUT /products/{id}`
    pub(crate) async fn update(
        &self,
        wire: Wire,
        Path(id): Path<String>,
        Body(request): Body<ProductUpdateRequestFactory>,
    ) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

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

        Ok(ApiResponse::ok(
            wire,
            ProductResponseFactory::of_domain(product.as_ref()),
        ))
    }

    /// `DELETE /products/{id}`
    pub(crate) async fn delete(&self, Path(id): Path<String>) -> Result<NoContent, ApiError> {
        let context = Session::require_user()?;

        self.products
            .delete(DeleteProductCommand { context, id })
            .await
            .map_err(ApiError::of_app)?;

        Ok(NoContent::new())
    }
}

/// O índice de classe de risco → o enum do domínio.
///
/// Os dois enums saem do mesmo `.fbs` e têm os mesmos índices, então a conversão
/// passa pelo número em vez de casar variante a variante — acrescentar uma
/// classe de risco não exige tocar aqui.
fn risk_class_of(value: Option<i32>) -> portmaster_app::domain::RiskClass {
    value
        .and_then(portmaster_app::domain::RiskClass::from_i32)
        .unwrap_or(portmaster_app::domain::RiskClass::None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    use crate::wire::tables as fbs;

    #[test]
    fn os_indices_dos_dois_enums_coincidem() {
        // Os dois saem do mesmo `.fbs`, mas por caminhos diferentes: um pelo
        // planus, outro escrito à mão no domain. Se divergirem, um produto
        // cadastrado como corrosivo vira radioativo em silêncio — e o índice é
        // justamente o que o DTO carrega, então é por ele que se compara.
        assert_eq!(
            risk_class_of(Some(i32::from(
                fbs::common::RiskClass::Class8CorrosiveSubstances as u8
            ))),
            portmaster_app::domain::RiskClass::Class8CorrosiveSubstances
        );
        assert_eq!(
            risk_class_of(Some(i32::from(fbs::common::RiskClass::None as u8))),
            portmaster_app::domain::RiskClass::None
        );
        assert_eq!(
            risk_class_of(Some(i32::from(
                fbs::common::RiskClass::Class1Explosives as u8
            ))),
            portmaster_app::domain::RiskClass::Class1Explosives
        );
    }

    #[test]
    fn um_campo_ausente_ou_fora_da_faixa_cai_no_neutro() {
        // Campo ausente não é erro nesta camada: vira o neutro, e é o
        // `TableModule` que decide se aquilo era obrigatório.
        assert_eq!(risk_class_of(None), portmaster_app::domain::RiskClass::None);
        assert_eq!(
            risk_class_of(Some(99)),
            portmaster_app::domain::RiskClass::None
        );
    }
}
