//! A orquestração de usuários.
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
use portmaster_domain::domain::User;
use portmaster_domain::table_modules::UserTM;
use portmaster_infra::query::views::{AccountView, UserListView};
use portmaster_infra::query::{dql, Dql as _, QueryRepository};
use portmaster_infra::repository::{RoleRepository, UserRepository, ViewCacheRepository};
use portmaster_infra::scope::{MasterScope, UnitOfWork};

use crate::commands::metadata::RegisterPermissionCommand;
use crate::commands::user::CreateUserCommand;
use crate::commands::user::DeleteUserCommand;
use crate::commands::user::ResetUserPasswordCommand;
use crate::commands::user::UpdateUserCommand;
use crate::commands::user::UpdateUserRolesCommand;
use crate::error::{AppError, UserError};
use crate::queries::user::GetUserQuery;
use crate::queries::user::ListUsersQuery;
use crate::services::MetadataService;
use crate::services::UserService;

/// Redefinir a senha de outro usuário.
const CHANGE_PASSWORD: &str = "user:change-password";
/// Cadastrar um usuário.
const CREATE: &str = "user:create";
/// Remover um usuário.
const DELETE: &str = "user:delete";
/// Ler um usuário.
const GET: &str = "user:get";
/// Listar usuários.
const LIST: &str = "user:list";
/// Alterar um usuário.
const UPDATE: &str = "user:update";
/// Trocar os papéis de um usuário.
const UPDATE_ROLES: &str = "user:update-roles";

/// O prefixo de toda leitura deste serviço — é o que uma escrita derruba.
const CACHE_GROUP: &str = "user";

/// A implementação, genérica sobre os ports que consome.
#[derive(Clone)]
pub(crate) struct UserServiceImpl<UR, RR, T, Q, C> {
    /// Persistência de usuários.
    users: UR,
    /// Persistência de papéis.
    roles: RR,
    /// As regras de usuário — quem constrói e valida.
    user_tm: T,
    /// Quem executa um DQL contra o banco.
    queries: Q,
    /// O cache do lado de leitura.
    views: C,
}

impl<UR, RR, T, Q, C> UserServiceImpl<UR, RR, T, Q, C> {
    /// Monta o caso de uso.
    pub(crate) const fn new(users: UR, roles: RR, user_tm: T, queries: Q, views: C) -> Self {
        Self {
            users,
            roles,
            user_tm,
            queries,
            views,
        }
    }
}

impl<UR, RR, T, Q, C> UserServiceImpl<UR, RR, T, Q, C>
where
    RR: RoleRepository + Send + Sync,
{
    /// Carrega os papéis pedidos, recusando um id que não existe.
    ///
    /// O PHP sincronizava os ids direto e deixava a chave estrangeira reclamar —
    /// o que devolvia erro de banco para o que é, na verdade, um id errado no
    /// corpo do request. Carregar antes transforma isso num 404 com o id que
    /// falhou.
    async fn resolve_roles(&self, ids: &[String]) -> Result<Vec<Box<dyn Role>>, UserError> {
        let mut roles = Vec::with_capacity(ids.len());

        for id in ids {
            let role = self
                .roles
                .find_by_id(id)
                .await?
                .ok_or_else(|| UserError::MissingRole(id.clone()))?;

            roles.push(role);
        }

        Ok(roles)
    }
}

impl<UR, RR, T, Q, C> UserService for UserServiceImpl<UR, RR, T, Q, C>
where
    UR: UserRepository + Send + Sync,
    RR: RoleRepository + Send + Sync,
    T: UserTM + Send + Sync,
    Q: QueryRepository + Send + Sync,
    C: ViewCacheRepository + Send + Sync,
{
    async fn declare_permissions<M: MetadataService + Send + Sync>(
        &self,
        registrar: &M,
    ) -> Result<(), UserError> {
        for slug in [
            CHANGE_PASSWORD,
            CREATE,
            DELETE,
            GET,
            LIST,
            UPDATE,
            UPDATE_ROLES,
        ] {
            registrar
                .register_permission(RegisterPermissionCommand {
                    slug: slug.to_owned(),
                })
                .await?;
        }

        Ok(())
    }

    /// Cria um usuário e liga os papéis dele.
    ///
    /// O e-mail é único, e a duplicidade é descoberta **aqui** em vez de deixar
    /// o índice reclamar: assim o cliente recebe um conflito com sentido, e não
    /// um erro de banco.
    ///
    /// Os papéis são uma tabela de ligação à parte, então `insert` grava a linha
    /// de `users` e o vínculo precisa do seu próprio comando.
    async fn create(&self, command: CreateUserCommand) -> Result<Box<dyn User>, UserError> {
        if !command.context.has_permission(CREATE) {
            return Err(AppError::permission_denied(CREATE).into());
        }

        let user = MasterScope::run(|uow| async move {
            if self.users.find_by_email(&command.email).await?.is_some() {
                return Err(UserError::EmailTaken);
            }

            let roles = self.resolve_roles(&command.role_ids).await?;
            let role_ids: Vec<String> = roles.iter().map(|role| role.id().to_owned()).collect();

            let user = self.user_tm.create(
                command.name,
                command.email,
                command.initial_password,
                roles,
            )?;

            self.users.insert(user.as_ref()).await?;

            self.users.sync_roles(user.id(), &role_ids).await?;

            uow.commit().await?;

            Ok(user)
        })
        .await?;

        self.views.invalidate(CACHE_GROUP).await?;

        Ok(user)
    }

    async fn update(&self, command: UpdateUserCommand) -> Result<Box<dyn User>, UserError> {
        if !command.context.has_permission(UPDATE) {
            return Err(AppError::permission_denied(UPDATE).into());
        }

        let user = MasterScope::run(|uow| async move {
            let Some(existing) = self.users.find_by_id(&command.id).await? else {
                return Err(UserError::Missing(command.id));
            };

            let updated = self
                .user_tm
                .update(existing.as_ref(), command.name, command.email)?;

            self.users.update(updated.as_ref()).await?;

            uow.commit().await?;

            Ok(updated)
        })
        .await?;

        self.views.invalidate(CACHE_GROUP).await?;

        Ok(user)
    }

    async fn update_roles(
        &self,
        command: UpdateUserRolesCommand,
    ) -> Result<Box<dyn User>, UserError> {
        if !command.context.has_permission(UPDATE_ROLES) {
            return Err(AppError::permission_denied(UPDATE_ROLES).into());
        }

        let user = MasterScope::run(|uow| async move {
            let Some(existing) = self.users.find_by_id(&command.id).await? else {
                return Err(UserError::Missing(command.id));
            };

            let roles = self.resolve_roles(&command.role_ids).await?;
            let role_ids: Vec<String> = roles.iter().map(|role| role.id().to_owned()).collect();

            let updated = self.user_tm.update_roles(existing.as_ref(), roles)?;

            self.users.sync_roles(&command.id, &role_ids).await?;

            uow.commit().await?;

            Ok(updated)
        })
        .await?;

        self.views.invalidate(CACHE_GROUP).await?;

        Ok(user)
    }

    /// Redefine a senha de outro usuário.
    ///
    /// **Sem pedir a senha atual**: quem redefine a senha de outro não a
    /// conhece. É a permissão que autoriza, não o conhecimento do segredo — ao
    /// contrário de `AccountService::change_password`, que rege a própria conta.
    async fn reset_password(&self, command: ResetUserPasswordCommand) -> Result<(), UserError> {
        if !command.context.has_permission(CHANGE_PASSWORD) {
            return Err(AppError::permission_denied(CHANGE_PASSWORD).into());
        }

        MasterScope::run(|uow| async move {
            let Some(existing) = self.users.find_by_id(&command.id).await? else {
                return Err(UserError::Missing(command.id));
            };

            let updated = self
                .user_tm
                .change_password(existing.as_ref(), command.new_password)?;

            self.users.update(updated.as_ref()).await?;

            uow.commit().await?;

            Ok(())
        })
        .await?;

        self.views.invalidate(CACHE_GROUP).await?;

        Ok(())
    }

    async fn delete(&self, command: DeleteUserCommand) -> Result<(), UserError> {
        if !command.context.has_permission(DELETE) {
            return Err(AppError::permission_denied(DELETE).into());
        }

        MasterScope::run(|uow| async move {
            if self.users.find_by_id(&command.id).await?.is_none() {
                return Err(UserError::Missing(command.id));
            }

            self.users.delete(&command.id).await?;

            uow.commit().await?;

            Ok(())
        })
        .await?;

        self.views.invalidate(CACHE_GROUP).await?;

        Ok(())
    }

    /// Um usuário com os papéis dele, pelo lado de leitura.
    ///
    /// Usa a mesma consulta de `GET /account`: um usuário com os papéis dele é o
    /// mesmo recorte, seja o próprio ou outro. A chave é outra — `user:` e não
    /// `account:` — porque é este serviço que a derruba.
    async fn get(&self, query: GetUserQuery) -> Result<AccountView, UserError> {
        if !query.context.has_permission(GET) {
            return Err(AppError::permission_denied(GET).into());
        }

        let dql = dql::get_account(&query.id)?;
        let key = dql.cache_key();

        if let Some(hit) = self.views.get(CACHE_GROUP, &key).await? {
            return Ok(hit);
        }

        let missing = query.id.clone();

        let view = MasterScope::run(|uow| async move {
            let Some(view) = self.queries.run(dql).await? else {
                return Err(UserError::Missing(missing));
            };

            uow.commit().await?;

            Ok(view)
        })
        .await?;

        // Falhar ao guardar não invalida a resposta: o cliente já tem o
        // dado correto, e o único prejuízo é o próximo pedido recalcular.
        self.views.put(CACHE_GROUP, &key, &view).await?;

        Ok(view)
    }

    async fn list(&self, query: ListUsersQuery) -> Result<UserListView, UserError> {
        if !query.context.has_permission(LIST) {
            return Err(AppError::permission_denied(LIST).into());
        }

        let dql = dql::list_users(query.page, query.limit);
        let key = dql.cache_key();

        if let Some(hit) = self.views.get(CACHE_GROUP, &key).await? {
            return Ok(hit);
        }

        let view = MasterScope::run(|uow| async move {
            let view = self.queries.run(dql).await?;

            uow.commit().await?;

            Ok::<_, UserError>(view)
        })
        .await?;

        // Falhar ao guardar não invalida a resposta: o cliente já tem o
        // dado correto, e o único prejuízo é o próximo pedido recalcular.
        self.views.put(CACHE_GROUP, &key, &view).await?;

        Ok(view)
    }
}

/// Os slugs deste serviço, para o teste do catálogo.
///
/// `cfg(test)`: em produção nada além deste arquivo vê um slug, e é isso que se
/// quer. O teste do catálogo precisa somá-los para afirmar as 25 permissões que
/// já existem em papéis gravados, e essa é a única razão de a lista existir.
#[cfg(test)]
pub(crate) const PERMISSIONS: &[&str] = &[
    CHANGE_PASSWORD,
    CREATE,
    DELETE,
    GET,
    LIST,
    UPDATE,
    UPDATE_ROLES,
];

#[cfg(test)]
#[path = "tests/user_service_impl_test.rs"]
mod tests;
