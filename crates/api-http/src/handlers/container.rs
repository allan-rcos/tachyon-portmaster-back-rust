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
use portmaster_app::container::{
    ContainerCommand, ContainerUseCase, CreateContainerCommand, GetContainerQuery,
    ListContainerSummariesQuery, ListContainersQuery, UpdateContainerCommand,
};
use portmaster_app::domain::ContainerStatus;

use super::{ContainerPageParams, SummaryPageParams};
use crate::error::{app_error_to_status, ApiError};
use crate::session::Session;
use crate::wire::http::{Accept, Body, Negotiated, NoContent};
use crate::wire::tables as fbs;
use crate::wire::view::container_of;

/// Os handlers de contêiner.
pub(crate) struct ContainerHandlers<C> {
    containers: C,
}

impl<C: ContainerUseCase> ContainerHandlers<C> {
    /// Monta os handlers.
    pub(crate) fn new(containers: C) -> Self {
        Self { containers }
    }

    /// `GET /containers`
    pub(crate) async fn list(
        &self,
        accept: Accept,
        Query(params): Query<ContainerPageParams>,
    ) -> Result<Negotiated<fbs::container::ContainerListResponse>, ApiError> {
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
            .map_err(app_error_to_status)?;

        Ok(Negotiated::ok(accept, view.into()))
    }

    /// `GET /containers/summary`
    ///
    /// Precisa estar registrada **antes** de `/containers/{id}` no router, senão
    /// `summary` casa como id.
    pub(crate) async fn summary(
        &self,
        accept: Accept,
        Query(params): Query<SummaryPageParams>,
    ) -> Result<Negotiated<fbs::container::ContainerSummaryListResponse>, ApiError> {
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
            .map_err(app_error_to_status)?;

        Ok(Negotiated::ok(accept, view.into()))
    }

    /// `POST /containers`
    pub(crate) async fn create(
        &self,
        accept: Accept,
        Body(request): Body<fbs::container::ContainerCreateRequest>,
    ) -> Result<Negotiated<fbs::container::ContainerResponse>, ApiError> {
        let context = Session::require_user()?;

        let container = self
            .containers
            .create(CreateContainerCommand {
                context,
                code: request.code.unwrap_or_default(),
                max_capacity: request.max_capacity,
            })
            .await
            .map_err(app_error_to_status)?;

        Ok(Negotiated::created(
            accept,
            container_of(container.as_ref()),
        ))
    }

    /// `GET /containers/{id}`
    pub(crate) async fn get(
        &self,
        accept: Accept,
        Path(id): Path<String>,
    ) -> Result<Negotiated<fbs::container::ContainerResponse>, ApiError> {
        let context = Session::require_user()?;

        let view = self
            .containers
            .get(GetContainerQuery { context, id })
            .await
            .map_err(app_error_to_status)?;

        Ok(Negotiated::ok(accept, view.into()))
    }

    /// `PUT /containers/{id}`
    pub(crate) async fn update(
        &self,
        accept: Accept,
        Path(id): Path<String>,
        Body(request): Body<fbs::container::ContainerUpdateRequest>,
    ) -> Result<Negotiated<fbs::container::ContainerResponse>, ApiError> {
        let context = Session::require_user()?;

        let container = self
            .containers
            .update(UpdateContainerCommand {
                context,
                id,
                max_capacity: request.max_capacity,
            })
            .await
            .map_err(app_error_to_status)?;

        Ok(Negotiated::ok(accept, container_of(container.as_ref())))
    }

    /// `DELETE /containers/{id}`
    pub(crate) async fn delete(&self, Path(id): Path<String>) -> Result<NoContent, ApiError> {
        let context = Session::require_user()?;

        self.containers
            .delete(ContainerCommand { context, id })
            .await
            .map_err(app_error_to_status)?;

        Ok(NoContent::new())
    }

    /// `POST /containers/{id}/seal`
    pub(crate) async fn seal(&self, Path(id): Path<String>) -> Result<NoContent, ApiError> {
        let context = Session::require_user()?;

        self.containers
            .seal(ContainerCommand { context, id })
            .await
            .map_err(app_error_to_status)?;

        Ok(NoContent::new())
    }

    /// `POST /containers/{id}/dispatch`
    pub(crate) async fn dispatch(&self, Path(id): Path<String>) -> Result<NoContent, ApiError> {
        let context = Session::require_user()?;

        self.containers
            .dispatch(ContainerCommand { context, id })
            .await
            .map_err(app_error_to_status)?;

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
