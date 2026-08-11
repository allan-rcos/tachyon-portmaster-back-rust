//! O contrato do controller de contêineres.

use crate::controllers::params::container_page_params::ContainerPageParams;
use crate::controllers::params::summary_page_params::SummaryPageParams;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::vo::container::container_create_x_request::ContainerCreateXRequest;
use crate::wire::vo::container::container_list_x_response::ContainerListXResponse;
use crate::wire::vo::container::container_summary_list_x_response::ContainerSummaryListXResponse;
use crate::wire::vo::container::container_update_x_request::ContainerUpdateXRequest;
use crate::wire::vo::container::container_x_response::ContainerXResponse;
use axum::extract::{Path, Query};

/// Os handlers de contêiner.
#[trait_variant::make(Send)]
pub(crate) trait ContainerController: Clone + Sync + 'static {
    /// `GET /containers`
    async fn list(self, params: Query<ContainerPageParams>) -> ApiResponse<ContainerListXResponse>;

    /// `GET /containers/summary`
    async fn summary(
        self,
        params: Query<SummaryPageParams>,
    ) -> ApiResponse<ContainerSummaryListXResponse>;

    /// `POST /containers`
    async fn create(
        self,
        request: Body<ContainerCreateXRequest>,
    ) -> ApiResponse<ContainerXResponse>;

    /// `GET /containers/{id}`
    async fn get(self, id: Path<String>) -> ApiResponse<ContainerXResponse>;

    /// `PUT /containers/{id}`
    async fn update(
        self,
        id: Path<String>,
        request: Body<ContainerUpdateXRequest>,
    ) -> ApiResponse<ContainerXResponse>;

    /// `DELETE /containers/{id}`
    async fn delete(self, id: Path<String>) -> ApiResponse;

    /// `POST /containers/{id}/seal`
    async fn seal(self, id: Path<String>) -> ApiResponse;

    /// `POST /containers/{id}/dispatch`
    async fn dispatch(self, id: Path<String>) -> ApiResponse;
}
