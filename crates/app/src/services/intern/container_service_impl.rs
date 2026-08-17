//! A orquestração de contêineres.
//!
//! ## As permissões são privadas
//!
//! Os slugs abaixo são **contrato**: já existem em papéis gravados no banco de
//! quem roda a versão PHP, e renomear qualquer um revoga silenciosamente o
//! acesso de quem o tinha. São `const` privadas porque uma permissão pertence a
//! exatamente um caso de uso — é ele quem a compara com o `UserContext`, e não
//! há segundo lugar no sistema que precise vê-la. O boot as registra chamando
//! `declare_permissions`, sem nunca lê-las.

use portmaster_domain::domain::Container;
use portmaster_domain::table_modules::ContainerTM;
use portmaster_infra::query::views::{
    ContainerListView, ContainerSummaryListView, ContainerViewItem,
};
use portmaster_infra::query::{dql, Dql as _, QueryRepository};
use portmaster_infra::repository::{ContainerRepository, ViewCacheRepository};
use portmaster_infra::scope::{MasterScope, UnitOfWork};

use crate::commands::container::ContainerCommand;
use crate::commands::container::CreateContainerCommand;
use crate::commands::container::UpdateContainerCommand;
use crate::commands::metadata::RegisterPermissionCommand;
use crate::error::{AppError, ContainerError};
use crate::event::{MetaEvent, MetaEventStackPublisher};
use crate::queries::container::GetContainerQuery;
use crate::queries::container::ListContainerSummariesQuery;
use crate::queries::container::ListContainersQuery;
use crate::services::ContainerService;
use crate::services::MetadataService;

/// Registrar um contêiner.
const CREATE: &str = "container:create";
/// Remover um contêiner.
const DELETE: &str = "container:delete";
/// Despachar um contêiner.
const DISPATCH: &str = "container:dispatch";
/// Ler contêineres.
const READ: &str = "container:read";
/// Selar um contêiner.
const SEAL: &str = "container:seal";
/// Ler o resumo de carga e telemetria.
const SUMMARY: &str = "container:summary";
/// Alterar um contêiner.
const UPDATE: &str = "container:update";

/// O prefixo das listagens deste serviço — é o que uma escrita derruba.
///
/// Cobre também o resumo de carga e telemetria: é leitura de contêiner, e sai
/// obsoleta pelas mesmas escritas.
const CACHE_GROUP: &str = "container";

/// Monta o caso de uso de contêiner.
///
/// Os ports chegam injetados e o que sai é o contrato: o tipo concreto não tem
/// nome fora deste arquivo, então nada além do provider consegue depender do
/// formato dele.
pub(crate) fn container_service<R, T, Q, C, E>(
    containers: R,
    container_tm: T,
    queries: Q,
    views: C,
    events: E,
) -> impl ContainerService + Sync + Clone + use<R, T, Q, C, E> + 'static
where
    R: ContainerRepository + Send + Sync + Clone + 'static,
    T: ContainerTM + Send + Sync + Clone + 'static,
    Q: QueryRepository + Send + Sync + Clone + 'static,
    C: ViewCacheRepository + Send + Sync + Clone + 'static,
    E: MetaEventStackPublisher + Send + Sync + Clone + 'static,
{
    ContainerServiceImpl {
        containers,
        container_tm,
        queries,
        views,
        events,
    }
}

/// A implementação, genérica sobre os ports que consome.
#[derive(Clone)]
struct ContainerServiceImpl<R, T, Q, C, E> {
    /// Persistência de contêineres.
    containers: R,
    /// As regras de contêiner.
    container_tm: T,
    /// Quem executa um DQL contra o banco.
    queries: Q,
    /// O cache do lado de leitura.
    views: C,
    /// Onde um acerto de cache é registrado, para quem quiser saber.
    events: E,
}

impl<R, T, Q, C, E> ContainerServiceImpl<R, T, Q, C, E>
where
    R: ContainerRepository + Send + Sync,
    T: ContainerTM + Send + Sync,
    C: ViewCacheRepository + Send + Sync,
{
    /// A moldura de selar e despachar: carrega, transita, grava.
    ///
    /// As duas só diferem no método do `TableModule` que aplicam, e é ele que
    /// recusa a transição inválida — o caso de uso não conhece nem o status
    /// exigido nem o mínimo de carga.
    async fn transition(
        &self,
        id: String,
        apply: impl Fn(&T, &dyn Container) -> Result<Box<dyn Container>, ContainerError>,
    ) -> Result<(), ContainerError> {
        MasterScope::run(|uow| async move {
            let Some(existing) = self.containers.find_by_id(&id).await? else {
                return Err(ContainerError::Missing(id));
            };

            let moved = apply(&self.container_tm, existing.as_ref())?;

            self.containers.update(moved.as_ref()).await?;

            uow.commit().await?;

            Ok(())
        })
        .await?;

        self.views.invalidate(CACHE_GROUP).await?;

        Ok(())
    }
}

impl<R, T, Q, C, E> ContainerService for ContainerServiceImpl<R, T, Q, C, E>
where
    R: ContainerRepository + Send + Sync,
    T: ContainerTM + Send + Sync,
    Q: QueryRepository + Send + Sync,
    C: ViewCacheRepository + Send + Sync,
    E: MetaEventStackPublisher + Send + Sync,
{
    async fn declare_permissions<M: MetadataService + Send + Sync>(
        &self,
        registrar: &M,
    ) -> Result<(), ContainerError> {
        for slug in [CREATE, DELETE, DISPATCH, READ, SEAL, SUMMARY, UPDATE] {
            registrar
                .register_permission(RegisterPermissionCommand {
                    slug: slug.to_owned(),
                })
                .await?;
        }

        Ok(())
    }

    async fn create(
        &self,
        command: CreateContainerCommand,
    ) -> Result<Box<dyn Container>, ContainerError> {
        if !command.context.has_permission(CREATE) {
            return Err(AppError::permission_denied(CREATE).into());
        }

        let container = MasterScope::run(|uow| async move {
            let container = self
                .container_tm
                .create(command.code, command.max_capacity)?;

            self.containers.insert(container.as_ref()).await?;

            uow.commit().await?;

            Ok::<_, ContainerError>(container)
        })
        .await?;

        self.views.invalidate(CACHE_GROUP).await?;

        Ok(container)
    }

    async fn update(
        &self,
        command: UpdateContainerCommand,
    ) -> Result<Box<dyn Container>, ContainerError> {
        if !command.context.has_permission(UPDATE) {
            return Err(AppError::permission_denied(UPDATE).into());
        }

        let container = MasterScope::run(|uow| async move {
            let Some(existing) = self.containers.find_by_id(&command.id).await? else {
                return Err(ContainerError::Missing(command.id));
            };

            let updated = self
                .container_tm
                .update(existing.as_ref(), command.max_capacity)?;

            self.containers.update(updated.as_ref()).await?;

            uow.commit().await?;

            Ok(updated)
        })
        .await?;

        self.views.invalidate(CACHE_GROUP).await?;

        Ok(container)
    }

    async fn delete(&self, command: ContainerCommand) -> Result<(), ContainerError> {
        if !command.context.has_permission(DELETE) {
            return Err(AppError::permission_denied(DELETE).into());
        }

        MasterScope::run(|uow| async move {
            if self.containers.find_by_id(&command.id).await?.is_none() {
                return Err(ContainerError::Missing(command.id));
            }

            self.containers.delete(&command.id).await?;

            uow.commit().await?;

            Ok(())
        })
        .await?;

        self.views.invalidate(CACHE_GROUP).await?;

        Ok(())
    }

    async fn seal(&self, command: ContainerCommand) -> Result<(), ContainerError> {
        if !command.context.has_permission(SEAL) {
            return Err(AppError::permission_denied(SEAL).into());
        }

        self.transition(command.id, |tm, container| Ok(tm.seal(container)?))
            .await
    }

    async fn dispatch(&self, command: ContainerCommand) -> Result<(), ContainerError> {
        if !command.context.has_permission(DISPATCH) {
            return Err(AppError::permission_denied(DISPATCH).into());
        }

        self.transition(command.id, |tm, container| Ok(tm.dispatch(container)?))
            .await
    }

    /// Um contêiner, direto da consulta — a leitura por id não passa pelo cache.
    async fn get(&self, query: GetContainerQuery) -> Result<ContainerViewItem, ContainerError> {
        if !query.context.has_permission(READ) {
            return Err(AppError::permission_denied(READ).into());
        }

        let dql = dql::get_container(&query.id)?;

        let missing = query.id.clone();

        let view = MasterScope::run(|uow| async move {
            let Some(view) = self.queries.run(dql).await? else {
                return Err(ContainerError::Missing(missing));
            };

            uow.commit().await?;

            Ok(view)
        })
        .await?;

        Ok(view)
    }

    async fn list(&self, query: ListContainersQuery) -> Result<ContainerListView, ContainerError> {
        if !query.context.has_permission(READ) {
            return Err(AppError::permission_denied(READ).into());
        }

        let dql = dql::list_containers(
            query.cursor.clone(),
            query.limit,
            query.search.as_deref(),
            query.status,
            query.status_in.clone(),
        );
        let key = dql.cache_key();

        if let Some(hit) = self.views.get(CACHE_GROUP, &key).await? {
            self.events.emit(MetaEvent::ViewCacheHit);
            return Ok(hit);
        }

        let view = MasterScope::run(|uow| async move {
            let view = self.queries.run(dql).await?;

            uow.commit().await?;

            Ok::<_, ContainerError>(view)
        })
        .await?;

        // Falhar ao guardar não invalida a resposta: o cliente já tem o
        // dado correto, e o único prejuízo é o próximo pedido recalcular.
        self.views.put(CACHE_GROUP, &key, &view).await?;

        Ok(view)
    }

    async fn list_summaries(
        &self,
        query: ListContainerSummariesQuery,
    ) -> Result<ContainerSummaryListView, ContainerError> {
        if !query.context.has_permission(SUMMARY) {
            return Err(AppError::permission_denied(SUMMARY).into());
        }

        let dql =
            dql::list_container_summaries(query.cursor.clone(), query.limit, query.id.clone())?;
        let key = dql.cache_key();

        if let Some(hit) = self.views.get(CACHE_GROUP, &key).await? {
            self.events.emit(MetaEvent::ViewCacheHit);
            return Ok(hit);
        }

        let view = MasterScope::run(|uow| async move {
            let view = self.queries.run(dql).await?;

            uow.commit().await?;

            Ok::<_, ContainerError>(view)
        })
        .await?;

        // Falhar ao guardar não invalida a resposta: o cliente já tem o
        // dado correto, e o único prejuízo é o próximo pedido recalcular.
        self.views.put(CACHE_GROUP, &key, &view).await?;

        Ok(view)
    }
}

#[cfg(test)]
#[path = "tests/container_service_impl_test.rs"]
mod tests;
