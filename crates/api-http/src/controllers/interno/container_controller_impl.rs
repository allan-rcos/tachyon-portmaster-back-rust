//! O controller de contêineres. Não sai do módulo.

use portmaster_app::commands::container::{
    ContainerCommand, CreateContainerCommand, UpdateContainerCommand,
};
use portmaster_app::context::UserContext;
use portmaster_app::domain::ContainerStatus;
use portmaster_app::queries::container::{
    GetContainerQuery, ListContainerSummariesQuery, ListContainersQuery,
};
use portmaster_app::services::ContainerUseCase;

use crate::controllers::container_controller::ContainerController;
use crate::controllers::params::container_page_params::ContainerPageParams;
use crate::controllers::params::summary_page_params::SummaryPageParams;
use crate::error::api_error::ApiError;
use crate::wire::vo::container::container_create_x_request::ContainerCreateXRequest;
use crate::wire::vo::container::container_list_x_response::ContainerListXResponse;
use crate::wire::vo::container::container_summary_list_x_response::ContainerSummaryListXResponse;
use crate::wire::vo::container::container_update_x_request::ContainerUpdateXRequest;
use crate::wire::vo::container::container_x_response::ContainerXResponse;

/// Os handlers de contêiner, genéricos sobre o caso de uso.
#[derive(Clone)]
pub(crate) struct ContainerControllerImpl<C> {
    /// O caso de uso de contêiner.
    containers: C,
}

impl<C: ContainerUseCase> ContainerControllerImpl<C> {
    /// Monta o controller.
    pub(crate) const fn new(containers: C) -> Self {
        Self { containers }
    }
}

impl<C: ContainerUseCase + Clone + Send + Sync + 'static> ContainerController
    for ContainerControllerImpl<C>
{
    async fn list(
        &self,
        context: UserContext,
        params: ContainerPageParams,
    ) -> Result<ContainerListXResponse, ApiError> {
        let view = self
            .containers
            .list(ListContainersQuery {
                context,
                cursor: params.cursor,
                limit: params.limit,
                search: params.search,
                status: params.status.as_deref().and_then(status_of),
                status_in: params
                    .status_in
                    .as_deref()
                    .map(|list| list.split(',').filter_map(status_of).collect())
                    .unwrap_or_default(),
            })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ContainerListXResponse::of(view))
    }

    async fn summary(
        &self,
        context: UserContext,
        params: SummaryPageParams,
    ) -> Result<ContainerSummaryListXResponse, ApiError> {
        let view = self
            .containers
            .list_summaries(ListContainerSummariesQuery {
                context,
                id: params.id.filter(|id| !id.is_empty()),
                cursor: params.cursor,
                limit: params.limit,
            })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ContainerSummaryListXResponse::of(view))
    }

    async fn create(
        &self,
        context: UserContext,
        request: ContainerCreateXRequest,
    ) -> Result<ContainerXResponse, ApiError> {
        let container = self
            .containers
            .create(CreateContainerCommand {
                context,
                code: request.code.unwrap_or_default(),
                max_capacity: request.max_capacity.unwrap_or_default(),
            })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ContainerXResponse::of_domain(container.as_ref()))
    }

    async fn get(&self, context: UserContext, id: String) -> Result<ContainerXResponse, ApiError> {
        let view = self
            .containers
            .get(GetContainerQuery { context, id })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ContainerXResponse::of(view))
    }

    async fn update(
        &self,
        context: UserContext,
        id: String,
        request: ContainerUpdateXRequest,
    ) -> Result<ContainerXResponse, ApiError> {
        let container = self
            .containers
            .update(UpdateContainerCommand {
                context,
                id,
                max_capacity: request.max_capacity.unwrap_or_default(),
            })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ContainerXResponse::of_domain(container.as_ref()))
    }

    async fn delete(&self, context: UserContext, id: String) -> Result<(), ApiError> {
        self.containers
            .delete(ContainerCommand { context, id })
            .await
            .map_err(ApiError::of_app)
    }

    async fn seal(&self, context: UserContext, id: String) -> Result<(), ApiError> {
        self.containers
            .seal(ContainerCommand { context, id })
            .await
            .map_err(ApiError::of_app)
    }

    async fn dispatch(&self, context: UserContext, id: String) -> Result<(), ApiError> {
        self.containers
            .dispatch(ContainerCommand { context, id })
            .await
            .map_err(ApiError::of_app)
    }
}

/// O status que o slug da querystring nomeia.
///
/// Slug desconhecido vira `None`, que o caso de uso lê como "sem filtro": um
/// parâmetro que não dá para interpretar não deveria esvaziar a listagem.
fn status_of(slug: &str) -> Option<ContainerStatus> {
    match slug.trim().to_ascii_lowercase().as_str() {
        "empty" => Some(ContainerStatus::Empty),
        "loading" => Some(ContainerStatus::Loading),
        "sealed" => Some(ContainerStatus::Sealed),
        "in-transit" => Some(ContainerStatus::InTransit),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn o_slug_vira_status() {
        assert_eq!(status_of("in-transit"), Some(ContainerStatus::InTransit));
        assert_eq!(status_of("  SEALED "), Some(ContainerStatus::Sealed));
    }

    /// Um filtro que não dá para interpretar não deveria esvaziar a listagem.
    #[test]
    fn slug_desconhecido_nao_filtra() {
        assert_eq!(status_of("carregando"), None);
    }
}
