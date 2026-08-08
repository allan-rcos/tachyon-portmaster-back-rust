//! A orquestração de contêineres.

use crate::cache::cache_key::CacheKey;
use crate::cache::invalidation::Invalidation;
use crate::cache::read_through::ReadThrough;
use crate::commands::container::ContainerCommand;
use crate::commands::container::CreateContainerCommand;
use crate::commands::container::UpdateContainerCommand;
use crate::error::AppError;
use crate::queries::container::GetContainerQuery;
use crate::queries::container::ListContainerSummariesQuery;
use crate::queries::container::ListContainersQuery;
use crate::security::requires_permission::RequiresPermission;
use crate::security::PermissionSlug;
use crate::services::ContainerUseCase;
use crate::transaction::transaction::Transaction;
use portmaster_domain::models::Container;
use portmaster_domain::table_modules::ContainerTM;
use portmaster_infra::cache::ReadCache;
use portmaster_infra::database::UnitOfWork;
use portmaster_infra::query::params::{ContainerListParams, SummaryListParams};
use portmaster_infra::query::views::{
    ContainerListView, ContainerSummaryListView, ContainerViewItem,
};
use portmaster_infra::query::{QueryFactory, QueryRepository};
use portmaster_infra::repository::ContainerRepository;

/// A implementação, genérica sobre os ports que consome.
pub(crate) struct ContainerUseCaseImpl<R, T, Q, F, C, U> {
    containers: R,
    container_tm: T,
    queries: Q,
    dqls: F,
    cache: C,
    unit_of_work: U,
    create_permission: RequiresPermission,
    update_permission: RequiresPermission,
    delete_permission: RequiresPermission,
    seal_permission: RequiresPermission,
    dispatch_permission: RequiresPermission,
    read_permission: RequiresPermission,
    summary_permission: RequiresPermission,
}

impl<R, T, Q, F, C, U> ContainerUseCaseImpl<R, T, Q, F, C, U> {
    /// Monta o caso de uso, declarando as permissões que ele exige.
    pub(crate) const fn new(
        containers: R,
        container_tm: T,
        queries: Q,
        dqls: F,
        cache: C,
        unit_of_work: U,
    ) -> Self {
        Self {
            containers,
            container_tm,
            queries,
            dqls,
            cache,
            unit_of_work,
            create_permission: RequiresPermission::new(PermissionSlug::CONTAINER_CREATE),
            update_permission: RequiresPermission::new(PermissionSlug::CONTAINER_UPDATE),
            delete_permission: RequiresPermission::new(PermissionSlug::CONTAINER_DELETE),
            seal_permission: RequiresPermission::new(PermissionSlug::CONTAINER_SEAL),
            dispatch_permission: RequiresPermission::new(PermissionSlug::CONTAINER_DISPATCH),
            read_permission: RequiresPermission::new(PermissionSlug::CONTAINER_READ),
            summary_permission: RequiresPermission::new(PermissionSlug::CONTAINER_SUMMARY),
        }
    }
}

impl<R, T, Q, F, C, U> ContainerUseCaseImpl<R, T, Q, F, C, U>
where
    R: ContainerRepository + Send + Sync,
    T: ContainerTM + Send + Sync,
    C: ReadCache + Send + Sync,
    U: UnitOfWork + Send + Sync,
{
    /// A moldura de selar e despachar: carrega, transita, grava.
    ///
    /// As duas só diferem no método do `TableModule` que aplicam, e é ele que
    /// recusa a transição inválida — o caso de uso não conhece nem o status
    /// exigido nem o mínimo de carga.
    async fn transition(
        &self,
        id: &str,
        apply: impl Fn(&T, &dyn Container) -> Result<Box<dyn Container>, AppError>,
    ) -> Result<(), AppError> {
        Transaction::run(&self.unit_of_work, async {
            let existing = self
                .containers
                .find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found("contêiner", id))?;

            let moved = apply(&self.container_tm, existing.as_ref())?;

            self.containers.update(moved.as_ref()).await?;

            Ok(())
        })
        .await?;

        ReadThrough::invalidate(&self.cache, Invalidation::CONTAINER_WRITE).await
    }
}

impl<R, T, Q, F, C, U> ContainerUseCase for ContainerUseCaseImpl<R, T, Q, F, C, U>
where
    R: ContainerRepository + Send + Sync,
    T: ContainerTM + Send + Sync,
    Q: QueryRepository + Send + Sync,
    F: QueryFactory + Send + Sync,
    C: ReadCache + Send + Sync,
    U: UnitOfWork + Send + Sync,
{
    async fn create(
        &self,
        command: CreateContainerCommand,
    ) -> Result<Box<dyn Container>, AppError> {
        self.create_permission.authorize(&command.context)?;

        let container = Transaction::run(&self.unit_of_work, async {
            let container = self
                .container_tm
                .create(command.code, command.max_capacity)?;

            self.containers.insert(container.as_ref()).await?;

            Ok(container)
        })
        .await?;

        ReadThrough::invalidate(&self.cache, Invalidation::CONTAINER_WRITE).await?;

        Ok(container)
    }

    async fn update(
        &self,
        command: UpdateContainerCommand,
    ) -> Result<Box<dyn Container>, AppError> {
        self.update_permission.authorize(&command.context)?;

        let container = Transaction::run(&self.unit_of_work, async {
            let existing = self
                .containers
                .find_by_id(&command.id)
                .await?
                .ok_or_else(|| AppError::not_found("contêiner", &command.id))?;

            let updated = self
                .container_tm
                .update(existing.as_ref(), command.max_capacity)?;

            self.containers.update(updated.as_ref()).await?;

            Ok(updated)
        })
        .await?;

        ReadThrough::invalidate(&self.cache, Invalidation::CONTAINER_WRITE).await?;

        Ok(container)
    }

    async fn delete(&self, command: ContainerCommand) -> Result<(), AppError> {
        self.delete_permission.authorize(&command.context)?;

        Transaction::run(&self.unit_of_work, async {
            self.containers
                .find_by_id(&command.id)
                .await?
                .ok_or_else(|| AppError::not_found("contêiner", &command.id))?;

            self.containers.delete(&command.id).await?;

            Ok(())
        })
        .await?;

        ReadThrough::invalidate(&self.cache, Invalidation::CONTAINER_WRITE).await
    }

    async fn seal(&self, command: ContainerCommand) -> Result<(), AppError> {
        self.seal_permission.authorize(&command.context)?;

        self.transition(&command.id, |tm, container| Ok(tm.seal(container)?))
            .await
    }

    async fn dispatch(&self, command: ContainerCommand) -> Result<(), AppError> {
        self.dispatch_permission.authorize(&command.context)?;

        self.transition(&command.id, |tm, container| Ok(tm.dispatch(container)?))
            .await
    }

    async fn get(&self, query: GetContainerQuery) -> Result<ContainerViewItem, AppError> {
        self.read_permission.authorize(&query.context)?;

        let key = CacheKey::of(CacheKey::CONTAINER, "get", &[&query.id]);

        ReadThrough::cached(&self.cache, &key, async {
            let dql = self.dqls.get_container(&query.id)?;

            Transaction::run(&self.unit_of_work, async {
                self.queries
                    .run(dql)
                    .await?
                    .ok_or_else(|| AppError::not_found("contêiner", &query.id))
            })
            .await
        })
        .await
    }

    async fn list(&self, query: ListContainersQuery) -> Result<ContainerListView, AppError> {
        self.read_permission.authorize(&query.context)?;

        let status_in = query
            .status_in
            .iter()
            .map(|status| status.as_i32().to_string())
            .collect::<Vec<_>>()
            .join(",");

        let key = CacheKey::of(
            CacheKey::CONTAINER,
            "list",
            &[
                &query.limit.unwrap_or_default().to_string(),
                query.cursor.as_deref().unwrap_or_default(),
                query.search.as_deref().unwrap_or_default(),
                &query
                    .status
                    .map(|status| status.as_i32().to_string())
                    .unwrap_or_default(),
                &status_in,
            ],
        );

        ReadThrough::cached(&self.cache, &key, async {
            let dql = self.dqls.list_containers(ContainerListParams {
                cursor: query.cursor.clone(),
                limit: query.limit,
                search: query.search.clone(),
                status: query.status,
                status_in: query.status_in.clone(),
            });

            Transaction::run(&self.unit_of_work, async {
                Ok(self.queries.run(dql).await?)
            })
            .await
        })
        .await
    }

    async fn list_summaries(
        &self,
        query: ListContainerSummariesQuery,
    ) -> Result<ContainerSummaryListView, AppError> {
        self.summary_permission.authorize(&query.context)?;

        let key = CacheKey::of(
            CacheKey::CONTAINER,
            "summary",
            &[
                &query.limit.unwrap_or_default().to_string(),
                query.cursor.as_deref().unwrap_or_default(),
                query.id.as_deref().unwrap_or_default(),
            ],
        );

        ReadThrough::cached(&self.cache, &key, async {
            let dql = self.dqls.list_container_summaries(SummaryListParams {
                id: query.id.clone(),
                cursor: query.cursor.clone(),
                limit: query.limit,
            })?;

            Transaction::run(&self.unit_of_work, async {
                Ok(self.queries.run(dql).await?)
            })
            .await
        })
        .await
    }
}
