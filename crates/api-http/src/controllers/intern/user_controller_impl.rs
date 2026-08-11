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
use crate::middleware::session_port::SessionPort;
use crate::ports::error::api_error::ApiError;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::vo::admin::user_admin_password_reset_x_request::UserAdminPasswordResetXRequest;
use crate::wire::vo::admin::user_admin_x_response::UserAdminXResponse;
use crate::wire::vo::admin::user_create_x_request::UserCreateXRequest;
use crate::wire::vo::admin::user_list_x_response::UserListXResponse;
use crate::wire::vo::admin::user_roles_update_x_request::UserRolesUpdateXRequest;
use crate::wire::vo::admin::user_update_x_request::UserUpdateXRequest;
use axum::extract::{Path, Query};

/// Os handlers de usuário, genéricos sobre o caso de uso.
#[derive(Clone)]
pub(crate) struct UserControllerImpl<U, S> {
    /// O caso de uso de usuário.
    users: U,
    /// Quem diz se há sessão, e quem a apresenta.
    session: S,
}

impl<U: UserUseCase, S: SessionPort> UserControllerImpl<U, S> {
    /// Monta o controller.
    pub(crate) const fn new(users: U, session: S) -> Self {
        Self { users, session }
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

impl<U: UserUseCase + Clone + Send + Sync + 'static, S: SessionPort> UserController
    for UserControllerImpl<U, S>
{
    async fn list(self, Query(params): Query<UserPageParams>) -> ApiResponse<UserListXResponse> {
        ApiResponse::ok(
            async {
                let context = self.session.require_user()?;

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
            .await,
        )
    }

    async fn create(
        self,
        Body(request): Body<UserCreateXRequest>,
    ) -> ApiResponse<UserAdminXResponse> {
        ApiResponse::created(
            async {
                let context = self.session.require_user()?;

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
            .await,
        )
    }

    async fn get(self, Path(id): Path<String>) -> ApiResponse<UserAdminXResponse> {
        ApiResponse::ok(
            async {
                let context = self.session.require_user()?;

                self.read(context, id).await
            }
            .await,
        )
    }

    async fn update(
        self,
        Path(id): Path<String>,
        Body(request): Body<UserUpdateXRequest>,
    ) -> ApiResponse<UserAdminXResponse> {
        ApiResponse::ok(
            async {
                let context = self.session.require_user()?;

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
            .await,
        )
    }

    /// Troca o conjunto de papéis do usuário.
    ///
    /// Ids vazios são descartados antes de chegar ao caso de uso: uma lista com
    /// `""` no meio é ruído de formulário, e recusar a requisição inteira por
    /// causa dele não ajudaria ninguém.
    async fn update_roles(
        self,
        Path(id): Path<String>,
        Body(request): Body<UserRolesUpdateXRequest>,
    ) -> ApiResponse<UserAdminXResponse> {
        ApiResponse::ok(
            async {
                let context = self.session.require_user()?;

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
            .await,
        )
    }

    async fn reset_password(
        self,
        Path(id): Path<String>,
        Body(request): Body<UserAdminPasswordResetXRequest>,
    ) -> ApiResponse {
        ApiResponse::no_content(
            async {
                let context = self.session.require_user()?;

                self.users
                    .reset_password(ResetUserPasswordCommand {
                        context,
                        id,
                        new_password: request.new_password.unwrap_or_default(),
                    })
                    .await
                    .map_err(to_api)
            }
            .await,
        )
    }

    async fn delete(self, Path(id): Path<String>) -> ApiResponse {
        ApiResponse::no_content(
            async {
                let context = self.session.require_user()?;

                self.users
                    .delete(DeleteUserCommand { context, id })
                    .await
                    .map_err(to_api)
            }
            .await,
        )
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
