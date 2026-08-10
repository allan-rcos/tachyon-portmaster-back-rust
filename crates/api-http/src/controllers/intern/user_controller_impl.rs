//! O controller de usuários. Não sai do módulo.

use axum::http::StatusCode;
use portmaster_app::commands::user::{
    CreateUserCommand, DeleteUserCommand, ResetUserPasswordCommand, UpdateUserCommand,
    UpdateUserRolesCommand,
};
use portmaster_app::context::UserContext;
use portmaster_app::error::UserError;
use portmaster_app::queries::user::{GetUserQuery, ListUsersQuery};
use portmaster_app::services::UserUseCase;

use crate::controllers::params::user_page_params::UserPageParams;
use crate::controllers::user_controller::UserController;
use crate::error::api_error::ApiError;
use crate::wire::vo::admin::user_admin_password_reset_x_request::UserAdminPasswordResetXRequest;
use crate::wire::vo::admin::user_admin_x_response::UserAdminXResponse;
use crate::wire::vo::admin::user_create_x_request::UserCreateXRequest;
use crate::wire::vo::admin::user_list_x_response::UserListXResponse;
use crate::wire::vo::admin::user_roles_update_x_request::UserRolesUpdateXRequest;
use crate::wire::vo::admin::user_update_x_request::UserUpdateXRequest;

/// Os handlers de usuário, genéricos sobre o caso de uso.
#[derive(Clone)]
pub(crate) struct UserControllerImpl<U> {
    /// O caso de uso de usuário.
    users: U,
}

impl<U: UserUseCase> UserControllerImpl<U> {
    /// Monta o controller.
    pub(crate) const fn new(users: U) -> Self {
        Self { users }
    }

    /// O usuário pelo lado de leitura, já na forma do fio.
    async fn read(&self, context: UserContext, id: String) -> Result<UserAdminXResponse, ApiError> {
        let view = self
            .users
            .get(GetUserQuery { context, id })
            .await
            .map_err(to_api)?;

        Ok(UserAdminXResponse::of(view))
    }
}

impl<U: UserUseCase + Clone + Send + Sync + 'static> UserController for UserControllerImpl<U> {
    async fn list(
        &self,
        context: UserContext,
        params: UserPageParams,
    ) -> Result<UserListXResponse, ApiError> {
        let view = self
            .users
            .list(ListUsersQuery {
                context,
                page: params.page,
                limit: params.limit,
            })
            .await
            .map_err(to_api)?;

        Ok(UserListXResponse::of(view))
    }

    async fn create(
        &self,
        context: UserContext,
        request: UserCreateXRequest,
    ) -> Result<UserAdminXResponse, ApiError> {
        let user = self
            .users
            .create(CreateUserCommand {
                context: context.clone(),
                name: request.name.unwrap_or_default(),
                email: request.email.unwrap_or_default(),
                initial_password: request.initial_password.unwrap_or_default(),
                role_ids: request.role_ids.unwrap_or_default(),
            })
            .await
            .map_err(to_api)?;

        self.read(context, user.id().to_owned()).await
    }

    async fn get(&self, context: UserContext, id: String) -> Result<UserAdminXResponse, ApiError> {
        self.read(context, id).await
    }

    async fn update(
        &self,
        context: UserContext,
        id: String,
        request: UserUpdateXRequest,
    ) -> Result<UserAdminXResponse, ApiError> {
        let user = self
            .users
            .update(UpdateUserCommand {
                context: context.clone(),
                id,
                name: request.name.unwrap_or_default(),
                email: request.email.unwrap_or_default(),
            })
            .await
            .map_err(to_api)?;

        self.read(context, user.id().to_owned()).await
    }

    /// Troca o conjunto de papéis do usuário.
    ///
    /// Ids vazios são descartados antes de chegar ao caso de uso: uma lista com
    /// `""` no meio é ruído de formulário, e recusar a requisição inteira por
    /// causa dele não ajudaria ninguém.
    async fn update_roles(
        &self,
        context: UserContext,
        id: String,
        request: UserRolesUpdateXRequest,
    ) -> Result<UserAdminXResponse, ApiError> {
        let user = self
            .users
            .update_roles(UpdateUserRolesCommand {
                context: context.clone(),
                id,
                role_ids: request
                    .role_ids
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|id| !id.is_empty())
                    .collect(),
            })
            .await
            .map_err(to_api)?;

        self.read(context, user.id().to_owned()).await
    }

    async fn reset_password(
        &self,
        context: UserContext,
        id: String,
        request: UserAdminPasswordResetXRequest,
    ) -> Result<(), ApiError> {
        self.users
            .reset_password(ResetUserPasswordCommand {
                context,
                id,
                new_password: request.new_password.unwrap_or_default(),
            })
            .await
            .map_err(to_api)
    }

    async fn delete(&self, context: UserContext, id: String) -> Result<(), ApiError> {
        self.users
            .delete(DeleteUserCommand { context, id })
            .await
            .map_err(to_api)
    }
}

/// Traduz a recusa do serviço de usuários no status que o cliente recebe.
///
/// O papel inexistente é 404 como o usuário, mas nomeia o outro recurso: o id
/// errado veio de um campo do corpo, não da rota, e quem lê a resposta precisa
/// saber qual dos dois procurar. E-mail já em uso é 409 — o pedido está bem
/// formado, o cadastro é que já tem aquele endereço.
fn to_api(error: UserError) -> ApiError {
    match error {
        UserError::Missing(id) => {
            ApiError::new(StatusCode::NOT_FOUND, format!("User {id} was not found."))
        }
        UserError::MissingRole(id) => {
            ApiError::new(StatusCode::NOT_FOUND, format!("Role {id} was not found."))
        }
        UserError::EmailTaken => ApiError::new(StatusCode::CONFLICT, error.to_string()),
        UserError::App(shared) => ApiError::of_app(shared),
    }
}
