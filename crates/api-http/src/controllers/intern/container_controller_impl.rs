//! O controller de contêineres. Não sai do módulo.

use axum::http::StatusCode;
use portmaster_app::commands::container::{
    ContainerCommand, CreateContainerCommand, UpdateContainerCommand,
};
use portmaster_app::domain::ContainerStatus;
use portmaster_app::error::ContainerError;
use portmaster_app::queries::container::{
    GetContainerQuery, ListContainerSummariesQuery, ListContainersQuery,
};
use portmaster_app::services::ContainerService;

use crate::controllers::container_controller::ContainerController;
use crate::controllers::params::container_page_params::ContainerPageParams;
use crate::controllers::params::summary_page_params::SummaryPageParams;
use crate::middleware::session_port::SessionPort;
use crate::ports::error::api_error::ApiError;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::vo::container::container_create_x_request::ContainerCreateXRequest;
use crate::wire::vo::container::container_list_x_response::ContainerListXResponse;
use crate::wire::vo::container::container_summary_list_x_response::ContainerSummaryListXResponse;
use crate::wire::vo::container::container_update_x_request::ContainerUpdateXRequest;
use crate::wire::vo::container::container_x_response::ContainerXResponse;
use axum::extract::{Path, Query};

/// Monta o controller de contêiner.
///
/// O service e o acesso à sessão chegam injetados, e o que sai é o contrato: o
/// tipo concreto não tem nome fora deste arquivo.
pub(crate) fn container_controller<C, S>(
    containers: C,
    session: S,
) -> impl ContainerController + use<C, S> + 'static
where
    C: ContainerService + Clone + Send + Sync + 'static,
    S: SessionPort + Clone + Send + Sync + 'static,
{
    ContainerControllerImpl {
        containers,
        session,
    }
}

/// Os handlers de contêiner, genéricos sobre o service.
#[derive(Clone)]
struct ContainerControllerImpl<C, S> {
    /// O service de contêiner.
    containers: C,
    /// Quem diz se há sessão, e quem a apresenta.
    session: S,
}

impl<C: ContainerService + Clone + Send + Sync + 'static, S: SessionPort> ContainerController
    for ContainerControllerImpl<C, S>
{
    async fn list(
        self,
        Query(params): Query<ContainerPageParams>,
    ) -> ApiResponse<ContainerListXResponse> {
        ApiResponse::ok(
            async {
                let context = self.session.require_user()?;

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
                    .map_err(to_api)?;

                Ok(ContainerListXResponse::of(view))
            }
            .await,
        )
    }

    async fn summary(
        self,
        Query(params): Query<SummaryPageParams>,
    ) -> ApiResponse<ContainerSummaryListXResponse> {
        ApiResponse::ok(
            async {
                let context = self.session.require_user()?;

                let view = self
                    .containers
                    .list_summaries(ListContainerSummariesQuery {
                        context,
                        id: params.id.filter(|id| !id.is_empty()),
                        cursor: params.cursor,
                        limit: params.limit,
                    })
                    .await
                    .map_err(to_api)?;

                Ok(ContainerSummaryListXResponse::of(view))
            }
            .await,
        )
    }

    async fn create(
        self,
        Body(request): Body<ContainerCreateXRequest>,
    ) -> ApiResponse<ContainerXResponse> {
        ApiResponse::created(
            async {
                let context = self.session.require_user()?;

                let container = self
                    .containers
                    .create(CreateContainerCommand {
                        context,
                        code: request.code.unwrap_or_default(),
                        max_capacity: request.max_capacity.unwrap_or_default(),
                    })
                    .await
                    .map_err(to_api)?;

                Ok(ContainerXResponse::of_domain(container.as_ref()))
            }
            .await,
        )
    }

    async fn get(self, Path(id): Path<String>) -> ApiResponse<ContainerXResponse> {
        ApiResponse::ok(
            async {
                let context = self.session.require_user()?;

                let view = self
                    .containers
                    .get(GetContainerQuery { context, id })
                    .await
                    .map_err(to_api)?;

                Ok(ContainerXResponse::of(view))
            }
            .await,
        )
    }

    async fn update(
        self,
        Path(id): Path<String>,
        Body(request): Body<ContainerUpdateXRequest>,
    ) -> ApiResponse<ContainerXResponse> {
        ApiResponse::ok(
            async {
                let context = self.session.require_user()?;

                let container = self
                    .containers
                    .update(UpdateContainerCommand {
                        context,
                        id,
                        max_capacity: request.max_capacity.unwrap_or_default(),
                    })
                    .await
                    .map_err(to_api)?;

                Ok(ContainerXResponse::of_domain(container.as_ref()))
            }
            .await,
        )
    }

    async fn delete(self, Path(id): Path<String>) -> ApiResponse {
        ApiResponse::no_content(
            async {
                let context = self.session.require_user()?;

                self.containers
                    .delete(ContainerCommand { context, id })
                    .await
                    .map_err(to_api)
            }
            .await,
        )
    }

    async fn seal(self, Path(id): Path<String>) -> ApiResponse {
        ApiResponse::no_content(
            async {
                let context = self.session.require_user()?;

                self.containers
                    .seal(ContainerCommand { context, id })
                    .await
                    .map_err(to_api)
            }
            .await,
        )
    }

    async fn dispatch(self, Path(id): Path<String>) -> ApiResponse {
        ApiResponse::no_content(
            async {
                let context = self.session.require_user()?;

                self.containers
                    .dispatch(ContainerCommand { context, id })
                    .await
                    .map_err(to_api)
            }
            .await,
        )
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

/// Traduz a recusa do serviço de contêineres no status que o cliente recebe.
///
/// Ausência é 404 e recusa de estado é 409: selar o que não está carregando não
/// é um pedido malformado, é o pátio que não está no ponto de aceitá-lo.
fn to_api(error: ContainerError) -> ApiError {
    match error {
        ContainerError::Missing(id) => ApiError::new(
            StatusCode::NOT_FOUND,
            format!("Container {id} was not found."),
        ),
        ContainerError::Refused(refused) => {
            ApiError::new(StatusCode::CONFLICT, refused.to_string())
        }
        ContainerError::App(shared) => ApiError::of_app(shared),
    }
}
