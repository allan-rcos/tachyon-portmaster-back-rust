//! `/containers` — o recurso central do pátio.
//!
//! Além do CRUD, tem as duas **transições**: selar e despachar. Elas são `POST`
//! sem corpo e respondem `204`, porque o que muda é o estado do contêiner e não
//! há nada a devolver que um `GET` não diga melhor.
//!
//! ## Os filtros de status chegam por slug
//!
//! `?status=loading` e `?status_in=empty,loading`. O que o cliente escreve é o
//! slug (minúsculo, com hífen), e não o nome da variante nem o índice: é o que a
//! API já publicava, e um índice numa querystring amarraria o cliente à ordem em
//! que o enum foi declarado.
//!
//! Um slug que o enum não reconhece é **descartado**, não recusado — inclusive
//! quando é o único. Filtrar por um status inexistente devolve a lista sem
//! aquele filtro, que é o que o PHP fazia; recusar com 422 transformaria um
//! parâmetro de conveniência em erro de integração.

use axum::extract::{Path, Query};
use portmaster_app::commands::container::ContainerCommand;
use portmaster_app::commands::container::CreateContainerCommand;
use portmaster_app::commands::container::UpdateContainerCommand;
use portmaster_app::domain::ContainerStatus;
use portmaster_app::queries::container::GetContainerQuery;
use portmaster_app::queries::container::ListContainerSummariesQuery;
use portmaster_app::queries::container::ListContainersQuery;
use portmaster_app::services::ContainerUseCase;

use crate::error::api_error::ApiError;
use crate::handlers::params::container_page_params::ContainerPageParams;
use crate::handlers::params::summary_page_params::SummaryPageParams;
use crate::session::Session;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::dto::container::container_create_request_factory::ContainerCreateRequestFactory;
use crate::wire::dto::container::container_list_response_factory::ContainerListResponseFactory;
use crate::wire::dto::container::container_response_factory::ContainerResponseFactory;
use crate::wire::dto::container::container_summary_list_response_factory::ContainerSummaryListResponseFactory;
use crate::wire::dto::container::container_update_request_factory::ContainerUpdateRequestFactory;
use crate::wire::no_content::NoContent;
use crate::wire::wire::Wire;

/// Os handlers de contêiner.
pub struct ContainerHandlers<C> {
    containers: C,
}

impl<C: ContainerUseCase> ContainerHandlers<C> {
    /// Monta os handlers.
    pub(crate) const fn new(containers: C) -> Self {
        Self { containers }
    }

    /// `GET /containers`
    pub(crate) async fn list(
        &self,
        wire: Wire,
        Query(params): Query<ContainerPageParams>,
    ) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

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
                    .map(status_list)
                    .unwrap_or_default(),
            })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ApiResponse::ok(
            wire,
            ContainerListResponseFactory::of(view),
        ))
    }

    /// `GET /containers/summary`
    ///
    /// Precisa estar registrada **antes** de `/containers/{id}` no router, senão
    /// `summary` casa como id.
    pub(crate) async fn summary(
        &self,
        wire: Wire,
        Query(params): Query<SummaryPageParams>,
    ) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

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

        Ok(ApiResponse::ok(
            wire,
            ContainerSummaryListResponseFactory::of(view),
        ))
    }

    /// `POST /containers`
    pub(crate) async fn create(
        &self,
        wire: Wire,
        Body(request): Body<ContainerCreateRequestFactory>,
    ) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

        let container = self
            .containers
            .create(CreateContainerCommand {
                context,
                code: request.code.unwrap_or_default(),
                max_capacity: request.max_capacity.unwrap_or_default(),
            })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ApiResponse::created(
            wire,
            ContainerResponseFactory::of_domain(container.as_ref()),
        ))
    }

    /// `GET /containers/{id}`
    pub(crate) async fn get(
        &self,
        wire: Wire,
        Path(id): Path<String>,
    ) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

        let view = self
            .containers
            .get(GetContainerQuery { context, id })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ApiResponse::ok(wire, ContainerResponseFactory::of(view)))
    }

    /// `PUT /containers/{id}`
    pub(crate) async fn update(
        &self,
        wire: Wire,
        Path(id): Path<String>,
        Body(request): Body<ContainerUpdateRequestFactory>,
    ) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

        let container = self
            .containers
            .update(UpdateContainerCommand {
                context,
                id,
                max_capacity: request.max_capacity.unwrap_or_default(),
            })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ApiResponse::ok(
            wire,
            ContainerResponseFactory::of_domain(container.as_ref()),
        ))
    }

    /// `DELETE /containers/{id}`
    pub(crate) async fn delete(&self, Path(id): Path<String>) -> Result<NoContent, ApiError> {
        let context = Session::require_user()?;

        self.containers
            .delete(ContainerCommand { context, id })
            .await
            .map_err(ApiError::of_app)?;

        Ok(NoContent::new())
    }

    /// `POST /containers/{id}/seal`
    pub(crate) async fn seal(&self, Path(id): Path<String>) -> Result<NoContent, ApiError> {
        let context = Session::require_user()?;

        self.containers
            .seal(ContainerCommand { context, id })
            .await
            .map_err(ApiError::of_app)?;

        Ok(NoContent::new())
    }

    /// `POST /containers/{id}/dispatch`
    pub(crate) async fn dispatch(&self, Path(id): Path<String>) -> Result<NoContent, ApiError> {
        let context = Session::require_user()?;

        self.containers
            .dispatch(ContainerCommand { context, id })
            .await
            .map_err(ApiError::of_app)?;

        Ok(NoContent::new())
    }
}

/// O status que um slug nomeia, ou `None` se nenhum.
///
/// Comparação sem diferenciar caixa porque o resto da API já normaliza texto
/// livre do mesmo jeito, e `?status=Loading` não é um pedido diferente de
/// `?status=loading`.
fn status_of(slug: &str) -> Option<ContainerStatus> {
    match slug.trim().to_ascii_lowercase().as_str() {
        "empty" => Some(ContainerStatus::Empty),
        "loading" => Some(ContainerStatus::Loading),
        "sealed" => Some(ContainerStatus::Sealed),
        "in-transit" => Some(ContainerStatus::InTransit),
        _ => None,
    }
}

/// Os status de uma lista separada por vírgula, descartando os desconhecidos.
fn status_list(list: &str) -> Vec<ContainerStatus> {
    list.split(',').filter_map(status_of).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn o_slug_e_o_que_o_cliente_escreve() {
        assert_eq!(status_of("in-transit"), Some(ContainerStatus::InTransit));
        assert_eq!(status_of(" Loading "), Some(ContainerStatus::Loading));
        assert_eq!(
            status_of("InTransit"),
            None,
            "o nome da variante não é o slug"
        );
        assert_eq!(status_of("1"), None, "o índice não atravessa a querystring");
    }

    #[test]
    fn a_lista_descarta_o_que_nao_reconhece_e_mantem_o_resto() {
        assert_eq!(
            status_list("empty,inexistente,sealed"),
            vec![ContainerStatus::Empty, ContainerStatus::Sealed]
        );
    }

    #[test]
    fn uma_lista_sem_nada_reconhecivel_nao_filtra_nada() {
        // Vazia e não "filtre por nada": o `app` trata a lista vazia como
        // ausência de filtro, que é o comportamento que a API já tinha.
        assert!(status_list("nada,disso,existe").is_empty());
    }
}
