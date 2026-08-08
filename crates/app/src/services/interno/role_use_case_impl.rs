//! A orquestração de papéis.

use crate::cache::cache_key::CacheKey;
use crate::cache::invalidation::Invalidation;
use crate::cache::read_through::ReadThrough;
use crate::commands::role::CreateRoleCommand;
use crate::commands::role::UpdateRolePermissionsCommand;
use crate::error::AppError;
use crate::queries::role::GetRoleQuery;
use crate::queries::role::ListRolesQuery;
use crate::security::requires_permission::RequiresPermission;
use crate::security::PermissionSlug;
use crate::services::RoleUseCase;
use crate::transaction::transaction::Transaction;
use portmaster_domain::models::Role;
use portmaster_domain::table_modules::RoleTM;
use portmaster_infra::cache::ReadCache;
use portmaster_infra::database::UnitOfWork;
use portmaster_infra::query::params::ListParams;
use portmaster_infra::query::views::{RoleListView, RoleViewItem};
use portmaster_infra::query::{QueryFactory, QueryRepository};
use portmaster_infra::repository::RoleRepository;

/// A implementação, genérica sobre os ports que consome.
pub(crate) struct RoleUseCaseImpl<R, T, Q, F, C, U> {
    /// Persistência de papéis.
    roles: R,
    /// As regras de papel.
    role_tm: T,
    /// Quem executa um DQL contra o banco.
    queries: Q,
    /// De onde os DQLs saem, já com os parâmetros.
    dqls: F,
    /// O cache de leitura, para o read-through e a invalidação.
    cache: C,
    /// Quem abre e fecha a transação.
    unit_of_work: U,
    /// A permissão exigida para create.
    create_permission: RequiresPermission,
    /// A permissão exigida para update.
    update_permission: RequiresPermission,
    /// A permissão exigida para read.
    read_permission: RequiresPermission,
}

impl<R, T, Q, F, C, U> RoleUseCaseImpl<R, T, Q, F, C, U> {
    /// Monta o caso de uso, declarando as permissões que ele exige.
    pub(crate) const fn new(
        roles: R,
        role_tm: T,
        queries: Q,
        dqls: F,
        cache: C,
        unit_of_work: U,
    ) -> Self {
        Self {
            roles,
            role_tm,
            queries,
            dqls,
            cache,
            unit_of_work,
            create_permission: RequiresPermission::new(PermissionSlug::ROLE_CREATE),
            update_permission: RequiresPermission::new(PermissionSlug::ROLE_UPDATE_PERMISSIONS),
            read_permission: RequiresPermission::new(PermissionSlug::ROLE_LIST),
        }
    }
}

impl<R, T, Q, F, C, U> RoleUseCase for RoleUseCaseImpl<R, T, Q, F, C, U>
where
    R: RoleRepository + Send + Sync,
    T: RoleTM + Send + Sync,
    Q: QueryRepository + Send + Sync,
    F: QueryFactory + Send + Sync,
    C: ReadCache + Send + Sync,
    U: UnitOfWork + Send + Sync,
{
    async fn create(&self, command: CreateRoleCommand) -> Result<Box<dyn Role>, AppError> {
        self.create_permission.authorize(&command.context)?;

        let role = Transaction::run(&self.unit_of_work, async {
            let role = self.role_tm.create(command.name, command.permissions)?;

            self.roles.insert(role.as_ref()).await?;

            Ok(role)
        })
        .await?;

        ReadThrough::invalidate(&self.cache, Invalidation::ROLE_WRITE).await?;

        Ok(role)
    }

    /// Substitui as permissões de um papel.
    ///
    /// A invalidação alcança `user:` e `account:` também: trocar as permissões
    /// de um papel muda o que toda conta que o carrega pode fazer.
    async fn update_permissions(
        &self,
        command: UpdateRolePermissionsCommand,
    ) -> Result<Box<dyn Role>, AppError> {
        self.update_permission.authorize(&command.context)?;

        let role = Transaction::run(&self.unit_of_work, async {
            let existing = self
                .roles
                .find_by_id(&command.id)
                .await?
                .ok_or_else(|| AppError::not_found("papel", &command.id))?;

            let updated = self
                .role_tm
                .update_permissions(existing.as_ref(), command.permissions)?;

            self.roles.update(updated.as_ref()).await?;

            Ok(updated)
        })
        .await?;

        ReadThrough::invalidate(&self.cache, Invalidation::ROLE_WRITE).await?;

        Ok(role)
    }

    async fn get(&self, query: GetRoleQuery) -> Result<RoleViewItem, AppError> {
        self.read_permission.authorize(&query.context)?;

        let key = CacheKey::of(CacheKey::ROLE, "get", &[&query.id]);

        ReadThrough::cached(&self.cache, &key, async {
            let dql = self.dqls.get_role(&query.id)?;

            Transaction::run(&self.unit_of_work, async {
                self.queries
                    .run(dql)
                    .await?
                    .ok_or_else(|| AppError::not_found("papel", &query.id))
            })
            .await
        })
        .await
    }

    async fn list(&self, query: ListRolesQuery) -> Result<RoleListView, AppError> {
        self.read_permission.authorize(&query.context)?;

        let key = CacheKey::of(
            CacheKey::ROLE,
            "list",
            &[
                &query.limit.unwrap_or_default().to_string(),
                query.cursor.as_deref().unwrap_or_default(),
                query.search.as_deref().unwrap_or_default(),
            ],
        );

        ReadThrough::cached(&self.cache, &key, async {
            let dql = self.dqls.list_roles(ListParams {
                cursor: query.cursor.clone(),
                limit: query.limit,
                search: query.search.clone(),
            });

            Transaction::run(&self.unit_of_work, async {
                Ok(self.queries.run(dql).await?)
            })
            .await
        })
        .await
    }
}
