//! A orquestração de usuários.

use crate::cache::cache_key::CacheKey;
use crate::cache::invalidation::Invalidation;
use crate::cache::read_through::ReadThrough;
use crate::commands::user::CreateUserCommand;
use crate::commands::user::DeleteUserCommand;
use crate::commands::user::ResetUserPasswordCommand;
use crate::commands::user::UpdateUserCommand;
use crate::commands::user::UpdateUserRolesCommand;
use crate::error::AppError;
use crate::queries::user::GetUserQuery;
use crate::queries::user::ListUsersQuery;
use crate::security::requires_permission::RequiresPermission;
use crate::security::PermissionSlug;
use crate::services::UserUseCase;
use crate::transaction::transaction::Transaction;
use portmaster_domain::models::Role;
use portmaster_domain::models::User;
use portmaster_domain::table_modules::UserTM;
use portmaster_infra::cache::ReadCache;
use portmaster_infra::database::UnitOfWork;
use portmaster_infra::query::params::UserListParams;
use portmaster_infra::query::views::{AccountView, UserListView};
use portmaster_infra::query::{QueryFactory, QueryRepository};
use portmaster_infra::repository::{RoleRepository, UserRepository};

/// A implementação, genérica sobre os ports que consome.
pub(crate) struct UserUseCaseImpl<UR, RR, T, Q, F, C, U> {
    users: UR,
    roles: RR,
    user_tm: T,
    queries: Q,
    dqls: F,
    cache: C,
    unit_of_work: U,
    create_permission: RequiresPermission,
    update_permission: RequiresPermission,
    update_roles_permission: RequiresPermission,
    change_password_permission: RequiresPermission,
    delete_permission: RequiresPermission,
    get_permission: RequiresPermission,
    list_permission: RequiresPermission,
}

impl<UR, RR, T, Q, F, C, U> UserUseCaseImpl<UR, RR, T, Q, F, C, U> {
    /// Monta o caso de uso, declarando as permissões que ele exige.
    pub(crate) const fn new(
        users: UR,
        roles: RR,
        user_tm: T,
        queries: Q,
        dqls: F,
        cache: C,
        unit_of_work: U,
    ) -> Self {
        Self {
            users,
            roles,
            user_tm,
            queries,
            dqls,
            cache,
            unit_of_work,
            create_permission: RequiresPermission::new(PermissionSlug::USER_CREATE),
            update_permission: RequiresPermission::new(PermissionSlug::USER_UPDATE),
            update_roles_permission: RequiresPermission::new(PermissionSlug::USER_UPDATE_ROLES),
            change_password_permission: RequiresPermission::new(
                PermissionSlug::USER_CHANGE_PASSWORD,
            ),
            delete_permission: RequiresPermission::new(PermissionSlug::USER_DELETE),
            get_permission: RequiresPermission::new(PermissionSlug::USER_GET),
            list_permission: RequiresPermission::new(PermissionSlug::USER_LIST),
        }
    }
}

impl<UR, RR, T, Q, F, C, U> UserUseCaseImpl<UR, RR, T, Q, F, C, U>
where
    RR: RoleRepository + Send + Sync,
{
    /// Carrega os papéis pedidos, recusando um id que não existe.
    ///
    /// O PHP sincronizava os ids direto e deixava a chave estrangeira reclamar —
    /// o que devolvia erro de banco para o que é, na verdade, um id errado no
    /// corpo do request. Carregar antes transforma isso num 404 com o id que
    /// falhou.
    async fn resolve_roles(&self, ids: &[String]) -> Result<Vec<Box<dyn Role>>, AppError> {
        let mut roles = Vec::with_capacity(ids.len());

        for id in ids {
            roles.push(
                self.roles
                    .find_by_id(id)
                    .await?
                    .ok_or_else(|| AppError::not_found("papel", id))?,
            );
        }

        Ok(roles)
    }
}

impl<UR, RR, T, Q, F, C, U> UserUseCase for UserUseCaseImpl<UR, RR, T, Q, F, C, U>
where
    UR: UserRepository + Send + Sync,
    RR: RoleRepository + Send + Sync,
    T: UserTM + Send + Sync,
    Q: QueryRepository + Send + Sync,
    F: QueryFactory + Send + Sync,
    C: ReadCache + Send + Sync,
    U: UnitOfWork + Send + Sync,
{
    async fn create(&self, command: CreateUserCommand) -> Result<Box<dyn User>, AppError> {
        self.create_permission.authorize(&command.context)?;

        let user = Transaction::run(&self.unit_of_work, async {
            // E-mail é único. Descobrir aqui devolve um conflito com sentido;
            // deixar o índice reclamar devolveria erro de banco.
            if self.users.find_by_email(&command.email).await?.is_some() {
                return Err(AppError::Conflict(
                    "A user with this e-mail already exists.".into(),
                ));
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
            // Os papéis são uma tabela de ligação à parte: `insert` grava a
            // linha de `users`, e o vínculo precisa do seu próprio comando.
            self.users.sync_roles(user.id(), &role_ids).await?;

            Ok(user)
        })
        .await?;

        ReadThrough::invalidate(&self.cache, Invalidation::USER_WRITE).await?;

        Ok(user)
    }

    async fn update(&self, command: UpdateUserCommand) -> Result<Box<dyn User>, AppError> {
        self.update_permission.authorize(&command.context)?;

        let user = Transaction::run(&self.unit_of_work, async {
            let existing = self
                .users
                .find_by_id(&command.id)
                .await?
                .ok_or_else(|| AppError::not_found("usuário", &command.id))?;

            let updated = self
                .user_tm
                .update(existing.as_ref(), command.name, command.email)?;

            self.users.update(updated.as_ref()).await?;

            Ok(updated)
        })
        .await?;

        ReadThrough::invalidate(&self.cache, Invalidation::USER_WRITE).await?;

        Ok(user)
    }

    async fn update_roles(
        &self,
        command: UpdateUserRolesCommand,
    ) -> Result<Box<dyn User>, AppError> {
        self.update_roles_permission.authorize(&command.context)?;

        let user = Transaction::run(&self.unit_of_work, async {
            let existing = self
                .users
                .find_by_id(&command.id)
                .await?
                .ok_or_else(|| AppError::not_found("usuário", &command.id))?;

            let roles = self.resolve_roles(&command.role_ids).await?;
            let role_ids: Vec<String> = roles.iter().map(|role| role.id().to_owned()).collect();

            let updated = self.user_tm.update_roles(existing.as_ref(), roles)?;

            self.users.sync_roles(&command.id, &role_ids).await?;

            Ok(updated)
        })
        .await?;

        ReadThrough::invalidate(&self.cache, Invalidation::USER_WRITE).await?;

        Ok(user)
    }

    async fn reset_password(&self, command: ResetUserPasswordCommand) -> Result<(), AppError> {
        self.change_password_permission
            .authorize(&command.context)?;

        Transaction::run(&self.unit_of_work, async {
            let existing = self
                .users
                .find_by_id(&command.id)
                .await?
                .ok_or_else(|| AppError::not_found("usuário", &command.id))?;

            // Sem pedir a senha atual: quem redefine a senha de outro não a
            // conhece. É a permissão que autoriza, não o conhecimento do segredo.
            let updated = self
                .user_tm
                .change_password(existing.as_ref(), command.new_password)?;

            self.users.update(updated.as_ref()).await?;

            Ok(())
        })
        .await?;

        ReadThrough::invalidate(&self.cache, Invalidation::USER_WRITE).await
    }

    async fn delete(&self, command: DeleteUserCommand) -> Result<(), AppError> {
        self.delete_permission.authorize(&command.context)?;

        Transaction::run(&self.unit_of_work, async {
            self.users
                .find_by_id(&command.id)
                .await?
                .ok_or_else(|| AppError::not_found("usuário", &command.id))?;

            self.users.delete(&command.id).await?;

            Ok(())
        })
        .await?;

        ReadThrough::invalidate(&self.cache, Invalidation::USER_WRITE).await
    }

    async fn get(&self, query: GetUserQuery) -> Result<AccountView, AppError> {
        self.get_permission.authorize(&query.context)?;

        let key = CacheKey::of(CacheKey::USER, "get", &[&query.id]);

        ReadThrough::cached(&self.cache, &key, async {
            // A mesma consulta de `GET /account`: um usuário com os papéis dele
            // é o mesmo recorte, seja o próprio ou outro.
            let dql = self.dqls.get_account(&query.id)?;

            Transaction::run(&self.unit_of_work, async {
                self.queries
                    .run(dql)
                    .await?
                    .ok_or_else(|| AppError::not_found("usuário", &query.id))
            })
            .await
        })
        .await
    }

    async fn list(&self, query: ListUsersQuery) -> Result<UserListView, AppError> {
        self.list_permission.authorize(&query.context)?;

        let key = CacheKey::of(
            CacheKey::USER,
            "list",
            &[
                &query.limit.unwrap_or_default().to_string(),
                &query.page.unwrap_or_default().to_string(),
            ],
        );

        ReadThrough::cached(&self.cache, &key, async {
            let dql = self.dqls.list_users(UserListParams {
                page: query.page,
                limit: query.limit,
            });

            Transaction::run(&self.unit_of_work, async {
                Ok(self.queries.run(dql).await?)
            })
            .await
        })
        .await
    }
}
