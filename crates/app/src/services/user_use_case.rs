//! Usuários.

use crate::commands::user::CreateUserCommand;
use crate::commands::user::DeleteUserCommand;
use crate::commands::user::ResetUserPasswordCommand;
use crate::commands::user::UpdateUserCommand;
use crate::commands::user::UpdateUserRolesCommand;
use crate::error::AppError;
use crate::queries::user::GetUserQuery;
use crate::queries::user::ListUsersQuery;
use portmaster_domain::models::User;
use portmaster_infra::query::views::{AccountView, UserListView};

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
