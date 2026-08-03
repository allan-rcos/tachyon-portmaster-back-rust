//! Os casos de uso de papel.

use portmaster_domain::role::{Role, RoleTM};
use portmaster_infra::cache::ReadCache;
use portmaster_infra::database::UnitOfWork;
use portmaster_infra::query::views::{RoleListView, RoleViewItem};
use portmaster_infra::query::{ListParams, QueryFactory, QueryRepository};
use portmaster_infra::repository::RoleRepository;

use crate::authorization::{slug, RequiresPermission};
use crate::cache::{self, prefix};
use crate::context::UserContext;
use crate::error::AppError;
use crate::transaction::transaction;

/// Criar um papel.
#[derive(Debug, Clone)]
pub struct CreateRoleCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Nome do papel.
    pub name: String,
    /// Os slugs de permissão que ele concede.
    pub permissions: Vec<String>,
}

/// Trocar as permissões de um papel.
#[derive(Debug, Clone)]
pub struct UpdateRolePermissionsCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Id do papel, em base62.
    pub id: String,
    /// O conjunto novo — substitui, não soma.
    pub permissions: Vec<String>,
}

/// Ler um papel.
#[derive(Debug, Clone)]
pub struct GetRoleQuery {
    /// Quem está consultando.
    pub context: UserContext,
    /// Id do papel, em base62.
    pub id: String,
}

/// Listar papéis.
#[derive(Debug, Clone)]
pub struct ListRolesQuery {
    /// Quem está consultando.
    pub context: UserContext,
    /// Token da página anterior.
    pub cursor: Option<String>,
    /// Tamanho da página.
    pub limit: Option<u32>,
    /// Termo de busca.
    pub search: Option<String>,
}

/// O que a apresentação pode pedir sobre papéis.
#[trait_variant::make(Send)]
pub trait RoleUseCase {
    /// Cria e devolve o papel.
    async fn create(&self, command: CreateRoleCommand) -> Result<Box<dyn Role>, AppError>;

    /// Substitui as permissões e devolve o papel atualizado.
    async fn update_permissions(
        &self,
        command: UpdateRolePermissionsCommand,
    ) -> Result<Box<dyn Role>, AppError>;

    /// Lê um papel.
    async fn get(&self, query: GetRoleQuery) -> Result<RoleViewItem, AppError>;

    /// Lista papéis.
    async fn list(&self, query: ListRolesQuery) -> Result<RoleListView, AppError>;
}

/// A implementação, genérica sobre os ports que consome.
pub(crate) struct RoleUseCaseImpl<R, T, Q, F, C, U> {
    roles: R,
    role_tm: T,
    queries: Q,
    dqls: F,
    cache: C,
    unit_of_work: U,
    create_permission: RequiresPermission,
    update_permission: RequiresPermission,
    read_permission: RequiresPermission,
}

impl<R, T, Q, F, C, U> RoleUseCaseImpl<R, T, Q, F, C, U> {
    /// Monta o caso de uso, declarando as permissões que ele exige.
    pub(crate) fn new(
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
            create_permission: RequiresPermission::new(slug::ROLE_CREATE),
            update_permission: RequiresPermission::new(slug::ROLE_UPDATE_PERMISSIONS),
            read_permission: RequiresPermission::new(slug::ROLE_LIST),
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

        let role = transaction(&self.unit_of_work, async {
            let role = self.role_tm.create(command.name, command.permissions)?;

            self.roles.insert(role.as_ref()).await?;

            Ok(role)
        })
        .await?;

        cache::invalidate(&self.cache, cache::ROLE_WRITE).await?;

        Ok(role)
    }

    async fn update_permissions(
        &self,
        command: UpdateRolePermissionsCommand,
    ) -> Result<Box<dyn Role>, AppError> {
        self.update_permission.authorize(&command.context)?;

        let role = transaction(&self.unit_of_work, async {
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

        // Trocar as permissões de um papel muda o que toda conta que o carrega
        // pode fazer — daí a invalidação alcançar `user:` e `account:` também.
        cache::invalidate(&self.cache, cache::ROLE_WRITE).await?;

        Ok(role)
    }

    async fn get(&self, query: GetRoleQuery) -> Result<RoleViewItem, AppError> {
        self.read_permission.authorize(&query.context)?;

        let key = cache::key(prefix::ROLE, "get", &[&query.id]);

        cache::cached(&self.cache, &key, async {
            let dql = self.dqls.get_role(&query.id)?;

            transaction(&self.unit_of_work, async {
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

        let key = cache::key(
            prefix::ROLE,
            "list",
            &[
                &query.limit.unwrap_or_default().to_string(),
                query.cursor.as_deref().unwrap_or_default(),
                query.search.as_deref().unwrap_or_default(),
            ],
        );

        cache::cached(&self.cache, &key, async {
            let dql = self.dqls.list_roles(ListParams {
                cursor: query.cursor.clone(),
                limit: query.limit,
                search: query.search.clone(),
            });

            transaction(&self.unit_of_work, async {
                Ok(self.queries.run(dql).await?)
            })
            .await
        })
        .await
    }
}
