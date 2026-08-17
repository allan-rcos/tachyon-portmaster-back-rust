//! A orquestração de papéis.
//!
//! ## As permissões são privadas
//!
//! Os slugs abaixo são **contrato**: já existem em papéis gravados no banco de
//! quem roda a versão PHP, e renomear qualquer um revoga silenciosamente o
//! acesso de quem o tinha. São `const` privadas porque uma permissão pertence a
//! exatamente um caso de uso — é ele quem a compara com o `UserContext`, e não
//! há segundo lugar no sistema que precise vê-la. O boot as registra chamando
//! `declare_permissions`, sem nunca lê-las.

use portmaster_domain::domain::Role;
use portmaster_domain::table_modules::RoleTM;
use portmaster_infra::query::views::{RoleListView, RoleViewItem};
use portmaster_infra::query::{dql, Dql as _, QueryRepository};
use portmaster_infra::repository::{RoleRepository, ViewCacheRepository};
use portmaster_infra::scope::{MasterScope, UnitOfWork};

use crate::commands::metadata::RegisterPermissionCommand;
use crate::commands::role::CreateRoleCommand;
use crate::commands::role::UpdateRolePermissionsCommand;
use crate::error::{AppError, RoleError};
use crate::event::{MetaEvent, MetaEventStackPublisher};
use crate::queries::role::GetRoleQuery;
use crate::queries::role::ListRolesQuery;
use crate::services::MetadataService;
use crate::services::RoleService;

/// Criar um papel.
const CREATE: &str = "role:create";
/// Ler papéis.
const LIST: &str = "role:list";
/// Trocar as permissões de um papel.
const UPDATE_PERMISSIONS: &str = "role:update-permissions";

/// O prefixo das listagens deste serviço — é o que uma escrita derruba.
///
/// Trocar as permissões de um papel muda o que toda conta que o carrega pode
/// fazer, e `user:`/`account:` **não** são derrubados por isso: cada serviço
/// invalida o que é seu, e o dado velho dos outros vive o TTL do cache.
const CACHE_GROUP: &str = "role";

/// Monta o caso de uso de papel.
///
/// Os ports chegam injetados e o que sai é o contrato: o tipo concreto não tem
/// nome fora deste arquivo, então nada além do provider consegue depender do
/// formato dele.
pub(crate) fn role_service<R, T, Q, C, E>(
    roles: R,
    role_tm: T,
    queries: Q,
    views: C,
    events: E,
) -> impl RoleService + Sync + Clone + use<R, T, Q, C, E> + 'static
where
    R: RoleRepository + Send + Sync + Clone + 'static,
    T: RoleTM + Send + Sync + Clone + 'static,
    Q: QueryRepository + Send + Sync + Clone + 'static,
    C: ViewCacheRepository + Send + Sync + Clone + 'static,
    E: MetaEventStackPublisher + Send + Sync + Clone + 'static,
{
    RoleServiceImpl {
        roles,
        role_tm,
        queries,
        views,
        events,
    }
}

/// A implementação, genérica sobre os ports que consome.
#[derive(Clone)]
struct RoleServiceImpl<R, T, Q, C, E> {
    /// Persistência de papéis.
    roles: R,
    /// As regras de papel.
    role_tm: T,
    /// Quem executa um DQL contra o banco.
    queries: Q,
    /// O cache do lado de leitura.
    views: C,
    /// Onde um acerto de cache é registrado, para quem quiser saber.
    events: E,
}

impl<R, T, Q, C, E> RoleService for RoleServiceImpl<R, T, Q, C, E>
where
    R: RoleRepository + Send + Sync,
    T: RoleTM + Send + Sync,
    Q: QueryRepository + Send + Sync,
    C: ViewCacheRepository + Send + Sync,
    E: MetaEventStackPublisher + Send + Sync,
{
    async fn declare_permissions<M: MetadataService + Send + Sync>(
        &self,
        registrar: &M,
    ) -> Result<(), RoleError> {
        for slug in [CREATE, LIST, UPDATE_PERMISSIONS] {
            registrar
                .register_permission(RegisterPermissionCommand {
                    slug: slug.to_owned(),
                })
                .await?;
        }

        Ok(())
    }

    async fn create(&self, command: CreateRoleCommand) -> Result<Box<dyn Role>, RoleError> {
        if !command.context.has_permission(CREATE) {
            return Err(AppError::permission_denied(CREATE).into());
        }

        let role = MasterScope::run(|uow| async move {
            let role = self.role_tm.create(command.name, command.permissions)?;

            self.roles.insert(role.as_ref()).await?;

            uow.commit().await?;

            Ok::<_, RoleError>(role)
        })
        .await?;

        self.views.invalidate(CACHE_GROUP).await?;

        Ok(role)
    }

    /// Substitui as permissões de um papel.
    async fn update_permissions(
        &self,
        command: UpdateRolePermissionsCommand,
    ) -> Result<Box<dyn Role>, RoleError> {
        if !command.context.has_permission(UPDATE_PERMISSIONS) {
            return Err(AppError::permission_denied(UPDATE_PERMISSIONS).into());
        }

        let role = MasterScope::run(|uow| async move {
            let Some(existing) = self.roles.find_by_id(&command.id).await? else {
                return Err(RoleError::Missing(command.id));
            };

            let updated = self
                .role_tm
                .update_permissions(existing.as_ref(), command.permissions)?;

            self.roles.update(updated.as_ref()).await?;

            uow.commit().await?;

            Ok(updated)
        })
        .await?;

        self.views.invalidate(CACHE_GROUP).await?;

        Ok(role)
    }

    /// Um papel, direto da consulta — a leitura por id não passa pelo cache.
    async fn get(&self, query: GetRoleQuery) -> Result<RoleViewItem, RoleError> {
        if !query.context.has_permission(LIST) {
            return Err(AppError::permission_denied(LIST).into());
        }

        let dql = dql::get_role(&query.id)?;

        let missing = query.id.clone();

        let view = MasterScope::run(|uow| async move {
            let Some(view) = self.queries.run(dql).await? else {
                return Err(RoleError::Missing(missing));
            };

            uow.commit().await?;

            Ok(view)
        })
        .await?;

        Ok(view)
    }

    async fn list(&self, query: ListRolesQuery) -> Result<RoleListView, RoleError> {
        if !query.context.has_permission(LIST) {
            return Err(AppError::permission_denied(LIST).into());
        }

        let dql = dql::list_roles(query.cursor.clone(), query.limit, query.search.as_deref());
        let key = dql.cache_key();

        if let Some(hit) = self.views.get(CACHE_GROUP, &key).await? {
            self.events.emit(MetaEvent::ViewCacheHit);
            return Ok(hit);
        }

        let view = MasterScope::run(|uow| async move {
            let view = self.queries.run(dql).await?;

            uow.commit().await?;

            Ok::<_, RoleError>(view)
        })
        .await?;

        // Falhar ao guardar não invalida a resposta: o cliente já tem o
        // dado correto, e o único prejuízo é o próximo pedido recalcular.
        self.views.put(CACHE_GROUP, &key, &view).await?;

        Ok(view)
    }
}

#[cfg(test)]
#[path = "tests/role_service_impl_test.rs"]
mod tests;
