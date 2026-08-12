//! Usuários.

use crate::commands::user::CreateUserCommand;
use crate::commands::user::DeleteUserCommand;
use crate::commands::user::ResetUserPasswordCommand;
use crate::commands::user::UpdateUserCommand;
use crate::commands::user::UpdateUserRolesCommand;
use crate::error::UserError;
use crate::queries::user::GetUserQuery;
use crate::queries::user::ListUsersQuery;
use crate::services::MetadataService;
use portmaster_domain::domain::User;
use portmaster_infra::query::views::{AccountView, UserListView};

/// O que a apresentação pode pedir sobre usuários.
#[trait_variant::make(Send)]
pub trait UserService {
    /// Registra, no boot, as permissões que este serviço exige.
    ///
    /// Os slugs são `const` privadas da implementação e **não** saem dela: quem
    /// os compara com o `UserContext` é o próprio caso de uso, e não há segundo
    /// lugar no sistema que precise vê-los. O que atravessa esta fronteira é a
    /// ação de registrar, nunca a lista — é o molde do `declarePermission` do
    /// PHP, onde a permissão pertence a exatamente um caso de uso.
    async fn declare_permissions<M: MetadataService + Send + Sync>(
        &self,
        registrar: &M,
    ) -> Result<(), UserError>;

    /// Cadastra e devolve o usuário criado.
    async fn create(&self, command: CreateUserCommand) -> Result<Box<dyn User>, UserError>;

    /// Altera e devolve o usuário atualizado.
    async fn update(&self, command: UpdateUserCommand) -> Result<Box<dyn User>, UserError>;

    /// Substitui os papéis e devolve o usuário atualizado.
    async fn update_roles(
        &self,
        command: UpdateUserRolesCommand,
    ) -> Result<Box<dyn User>, UserError>;

    /// Redefine a senha.
    async fn reset_password(&self, command: ResetUserPasswordCommand) -> Result<(), UserError>;

    /// Remove — soft-delete.
    async fn delete(&self, command: DeleteUserCommand) -> Result<(), UserError>;

    /// Lê um usuário com os papéis dele.
    async fn get(&self, query: GetUserQuery) -> Result<AccountView, UserError>;

    /// Lista usuários.
    async fn list(&self, query: ListUsersQuery) -> Result<UserListView, UserError>;
}
