//! `/products` — o molde dos demais recursos.
//!
//! Mostra os dois formatos de handler que se repetem no resto: os de **escrita**
//! (corpo → Command → objeto de domínio → tabela) e os de **leitura**
//! (querystring → Query → View → tabela).

use axum::extract::{Path, Query};
use portmaster_app::product::{
    CreateProductCommand, DeleteProductCommand, GetProductQuery, ListProductsQuery, ProductUseCase,
    UpdateProductCommand,
};

use super::PageParams;
use crate::error::{app_error_to_status, ApiError};
use crate::session::Session;
use crate::wire::http::{Accept, Body, Negotiated, NoContent};
use crate::wire::tables as fbs;
use crate::wire::view::product_of;

/// Os handlers de produto, sobre o caso de uso que o provider entregou.
pub(crate) struct ProductHandlers<U> {
    products: U,
}

impl<U: ProductUseCase> ProductHandlers<U> {
    /// Monta os handlers.
    pub(crate) fn new(products: U) -> Self {
        Self { products }
    }

    /// `GET /products`
    pub(crate) async fn list(
        &self,
        accept: Accept,
        Query(params): Query<PageParams>,
    ) -> Result<Negotiated<fbs::product::ProductListResponse>, ApiError> {
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
            .map_err(app_error_to_status)?;

        Ok(Negotiated::ok(accept, view.into()))
    }

    /// `POST /products`
    pub(crate) async fn create(
        &self,
        accept: Accept,
        Body(request): Body<fbs::product::ProductCreateRequest>,
    ) -> Result<Negotiated<fbs::product::ProductResponse>, ApiError> {
        let context = Session::require_user()?;

        let product = self
            .products
            .create(CreateProductCommand {
                context,
                name: request.name,
                density: request.density,
                // O enum do wire e o do domínio têm os mesmos índices — os dois
                // saem do mesmo `.fbs`. A conversão passa pelo índice em vez de
                // casar variante a variante, para que acrescentar uma classe de
                // risco não exija tocar aqui.
                risk_class: risk_class_of(request.risk_class),
            })
            .await
            .map_err(app_error_to_status)?;

        Ok(Negotiated::created(accept, product_of(product.as_ref())))
    }

    /// `GET /products/{id}`
    pub(crate) async fn get(
        &self,
        accept: Accept,
        Path(id): Path<String>,
    ) -> Result<Negotiated<fbs::product::ProductResponse>, ApiError> {
        let context = Session::require_user()?;

        let view = self
            .products
            .get(GetProductQuery { context, id })
            .await
            .map_err(app_error_to_status)?;

        Ok(Negotiated::ok(accept, view.into()))
    }

    /// `PUT /products/{id}`
    pub(crate) async fn update(
        &self,
        accept: Accept,
        Path(id): Path<String>,
        Body(request): Body<fbs::product::ProductUpdateRequest>,
    ) -> Result<Negotiated<fbs::product::ProductResponse>, ApiError> {
        let context = Session::require_user()?;

        let product = self
            .products
            .update(UpdateProductCommand {
                context,
                id,
                name: request.name,
                density: request.density,
                risk_class: risk_class_of(request.risk_class),
            })
            .await
            .map_err(app_error_to_status)?;

        Ok(Negotiated::ok(accept, product_of(product.as_ref())))
    }

    /// `DELETE /products/{id}`
    pub(crate) async fn delete(&self, Path(id): Path<String>) -> Result<NoContent, ApiError> {
        let context = Session::require_user()?;

        self.products
            .delete(DeleteProductCommand { context, id })
            .await
            .map_err(app_error_to_status)?;

        Ok(NoContent::new())
    }
}

/// O enum do wire → o do domínio, pelo índice.
fn risk_class_of(value: fbs::common::RiskClass) -> portmaster_app::domain::RiskClass {
    portmaster_app::domain::RiskClass::from_i32(i32::from(value as u8))
        .unwrap_or(portmaster_app::domain::RiskClass::None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn os_indices_dos_dois_enums_coincidem() {
        // Os dois saem do mesmo `.fbs`, mas por caminhos diferentes: um pelo
        // planus, outro escrito à mão no domain. Se divergirem, um produto
        // cadastrado como corrosivo vira radioativo em silêncio.
        assert_eq!(
            risk_class_of(fbs::common::RiskClass::Class8CorrosiveSubstances),
            portmaster_app::domain::RiskClass::Class8CorrosiveSubstances
        );
        assert_eq!(
            risk_class_of(fbs::common::RiskClass::None),
            portmaster_app::domain::RiskClass::None
        );
        assert_eq!(
            risk_class_of(fbs::common::RiskClass::Class1Explosives),
            portmaster_app::domain::RiskClass::Class1Explosives
        );
    }
}
