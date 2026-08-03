//! Os casos de uso de usuário — a administração de contas alheias.
//!
//! O que o usuário faz com a **própria** conta é [`crate::account`]: lá não há
//! permissão a exigir, porque agir sobre si mesmo é o que a sessão já autoriza.

use portmaster_domain::role::Role;
use portmaster_domain::user::{User, UserTM};
use portmaster_infra::cache::ReadCache;
use portmaster_infra::database::UnitOfWork;
use portmaster_infra::query::views::{AccountView, UserListView};
use portmaster_infra::query::{QueryFactory, QueryRepository, UserListParams};
use portmaster_infra::repository::{RoleRepository, UserRepository};

use crate::authorization::{slug, RequiresPermission};
use crate::cache::{self, prefix};
use crate::context::UserContext;
use crate::error::AppError;
use crate::transaction::transaction;

/// Cadastrar um usuário.
#[derive(Debug, Clone)]
pub struct CreateUserCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Nome do usuário.
    pub name: String,
    /// E-mail — único no sistema.
    pub email: String,
    /// Senha inicial.
    pub initial_password: String,
    /// Ids dos papéis a atribuir, em base62.
    pub role_ids: Vec<String>,
}

/// Alterar nome e e-mail de um usuário.
#[derive(Debug, Clone)]
pub struct UpdateUserCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Id do usuário, em base62.
    pub id: String,
    /// Nome do usuário.
    pub name: String,
    /// E-mail do usuário.
    pub email: String,
}

/// Trocar os papéis de um usuário.
#[derive(Debug, Clone)]
pub struct UpdateUserRolesCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Id do usuário, em base62.
    pub id: String,
    /// O conjunto novo — substitui, não soma.
    pub role_ids: Vec<String>,
}

/// Redefinir a senha de um usuário.
///
/// É o caminho administrativo: não pede a senha atual, porque quem a executa não
/// a conhece. Trocar a própria senha é [`crate::account::ChangePasswordCommand`],
/// que exige a atual.
#[derive(Debug, Clone)]
pub struct ResetUserPasswordCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Id do usuário, em base62.
    pub id: String,
    /// A senha nova.
    pub new_password: String,
}

/// Remover um usuário.
#[derive(Debug, Clone)]
pub struct DeleteUserCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Id do usuário, em base62.
    pub id: String,
}

/// Ler um usuário.
#[derive(Debug, Clone)]
pub struct GetUserQuery {
    /// Quem está consultando.
    pub context: UserContext,
    /// Id do usuário, em base62.
    pub id: String,
}

/// Listar usuários.
#[derive(Debug, Clone)]
pub struct ListUsersQuery {
    /// Quem está consultando.
    pub context: UserContext,
    /// Página, começando em 1.
    pub page: Option<u32>,
    /// Tamanho da página.
    pub limit: Option<u32>,
}

/// O que a apresentação pode pedir sobre usuários.
#[trait_variant::make(Send)]
pub trait UserUseCase {
    /// Cadastra e devolve o usuário criado.
    async fn create(&self, command: CreateUserCommand) -> Result<Box<dyn User>, AppError>;

    /// Altera e devolve o usuário atualizado.
    async fn update(&self, command: UpdateUserCommand) -> Result<Box<dyn User>, AppError>;

    /// Substitui os papéis e devolve o usuário atualizado.
    async fn update_roles(
        &self,
        command: UpdateUserRolesCommand,
    ) -> Result<Box<dyn User>, AppError>;

    /// Redefine a senha.
    async fn reset_password(&self, command: ResetUserPasswordCommand) -> Result<(), AppError>;

    /// Remove — soft-delete.
    async fn delete(&self, command: DeleteUserCommand) -> Result<(), AppError>;

    /// Lê um usuário com os papéis dele.
    async fn get(&self, query: GetUserQuery) -> Result<AccountView, AppError>;

    /// Lista usuários.
    async fn list(&self, query: ListUsersQuery) -> Result<UserListView, AppError>;
}

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
    pub(crate) fn new(
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
            create_permission: RequiresPermission::new(slug::USER_CREATE),
            update_permission: RequiresPermission::new(slug::USER_UPDATE),
            update_roles_permission: RequiresPermission::new(slug::USER_UPDATE_ROLES),
            change_password_permission: RequiresPermission::new(slug::USER_CHANGE_PASSWORD),
            delete_permission: RequiresPermission::new(slug::USER_DELETE),
            get_permission: RequiresPermission::new(slug::USER_GET),
            list_permission: RequiresPermission::new(slug::USER_LIST),
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

        let user = transaction(&self.unit_of_work, async {
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

        cache::invalidate(&self.cache, cache::USER_WRITE).await?;

        Ok(user)
    }

    async fn update(&self, command: UpdateUserCommand) -> Result<Box<dyn User>, AppError> {
        self.update_permission.authorize(&command.context)?;

        let user = transaction(&self.unit_of_work, async {
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

        cache::invalidate(&self.cache, cache::USER_WRITE).await?;

        Ok(user)
    }

    async fn update_roles(
        &self,
        command: UpdateUserRolesCommand,
    ) -> Result<Box<dyn User>, AppError> {
        self.update_roles_permission.authorize(&command.context)?;

        let user = transaction(&self.unit_of_work, async {
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

        cache::invalidate(&self.cache, cache::USER_WRITE).await?;

        Ok(user)
    }

    async fn reset_password(&self, command: ResetUserPasswordCommand) -> Result<(), AppError> {
        self.change_password_permission
            .authorize(&command.context)?;

        transaction(&self.unit_of_work, async {
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

        cache::invalidate(&self.cache, cache::USER_WRITE).await
    }

    async fn delete(&self, command: DeleteUserCommand) -> Result<(), AppError> {
        self.delete_permission.authorize(&command.context)?;

        transaction(&self.unit_of_work, async {
            self.users
                .find_by_id(&command.id)
                .await?
                .ok_or_else(|| AppError::not_found("usuário", &command.id))?;

            self.users.delete(&command.id).await?;

            Ok(())
        })
        .await?;

        cache::invalidate(&self.cache, cache::USER_WRITE).await
    }

    async fn get(&self, query: GetUserQuery) -> Result<AccountView, AppError> {
        self.get_permission.authorize(&query.context)?;

        let key = cache::key(prefix::USER, "get", &[&query.id]);

        cache::cached(&self.cache, &key, async {
            // A mesma consulta de `GET /account`: um usuário com os papéis dele
            // é o mesmo recorte, seja o próprio ou outro.
            let dql = self.dqls.get_account(&query.id)?;

            transaction(&self.unit_of_work, async {
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

        let key = cache::key(
            prefix::USER,
            "list",
            &[
                &query.limit.unwrap_or_default().to_string(),
                &query.page.unwrap_or_default().to_string(),
            ],
        );

        cache::cached(&self.cache, &key, async {
            let dql = self.dqls.list_users(UserListParams {
                page: query.page,
                limit: query.limit,
            });

            transaction(&self.unit_of_work, async {
                Ok(self.queries.run(dql).await?)
            })
            .await
        })
        .await
    }
}
