//! O contrato do controller de contêineres.

use portmaster_app::context::UserContext;

use crate::controllers::params::container_page_params::ContainerPageParams;
use crate::controllers::params::summary_page_params::SummaryPageParams;
use crate::error::api_error::ApiError;
use crate::wire::vo::container::container_create_x_request::ContainerCreateXRequest;
use crate::wire::vo::container::container_list_x_response::ContainerListXResponse;
use crate::wire::vo::container::container_summary_list_x_response::ContainerSummaryListXResponse;
use crate::wire::vo::container::container_update_x_request::ContainerUpdateXRequest;
use crate::wire::vo::container::container_x_response::ContainerXResponse;

/// Os handlers de contêiner.
#[trait_variant::make(Send)]
pub(crate) trait ContainerController: Clone + Sync + 'static {
    /// `GET /containers`
    async fn list(
        &self,
        context: UserContext,
        params: ContainerPageParams,
    ) -> Result<ContainerListXResponse, ApiError>;

    /// `GET /containers/summary`
    async fn summary(
        &self,
        context: UserContext,
        params: SummaryPageParams,
    ) -> Result<ContainerSummaryListXResponse, ApiError>;

    /// `POST /containers`
    async fn create(
        &self,
        context: UserContext,
        request: ContainerCreateXRequest,
    ) -> Result<ContainerXResponse, ApiError>;

    /// `GET /containers/{id}`
    async fn get(&self, context: UserContext, id: String) -> Result<ContainerXResponse, ApiError>;

    /// `PUT /containers/{id}`
    async fn update(
        &self,
        context: UserContext,
        id: String,
        request: ContainerUpdateXRequest,
    ) -> Result<ContainerXResponse, ApiError>;

    /// `DELETE /containers/{id}`
    async fn delete(&self, context: UserContext, id: String) -> Result<(), ApiError>;

    /// `POST /containers/{id}/seal`
    async fn seal(&self, context: UserContext, id: String) -> Result<(), ApiError>;

    /// `POST /containers/{id}/dispatch`
    async fn dispatch(&self, context: UserContext, id: String) -> Result<(), ApiError>;
}
