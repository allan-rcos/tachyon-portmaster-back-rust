//! `/users` — a administração de contas alheias.
//!
//! Cada operação exige a sua própria permissão, e nenhuma delas é a mesma que
//! rege `/account`: mudar o próprio nome e mudar o de outra pessoa são atos
//! diferentes, e o `app` os separa em permissões diferentes.
//!
//! ## A listagem pagina por página, não por cursor
//!
//! É a única do sistema assim. Contas são poucas e administradas à mão, com uma
//! tela que precisa de "página 3" e não de "próxima" — que é justamente o que um
//! cursor não sabe responder.
//!
//! ## Por que a escrita relê antes de responder
//!
//! `UserAdminResponse` traz os papéis expandidos, com nome e permissões. O
//! objeto que a escrita devolve tem os papéis, mas não os números que a consulta
//! calcula; reler pelo lado de leitura é o que faz a resposta da criação ser
//! idêntica ao `GET` que vem depois.

use axum::extract::{Path, Query};
use portmaster_app::commands::user::CreateUserCommand;
use portmaster_app::commands::user::DeleteUserCommand;
use portmaster_app::commands::user::ResetUserPasswordCommand;
use portmaster_app::commands::user::UpdateUserCommand;
use portmaster_app::commands::user::UpdateUserRolesCommand;
use portmaster_app::context::UserContext;
use portmaster_app::queries::user::GetUserQuery;
use portmaster_app::queries::user::ListUsersQuery;
use portmaster_app::services::UserUseCase;

use crate::error::api_error::ApiError;
use crate::handlers::params::user_page_params::UserPageParams;
use crate::session::Session;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::dto::admin::role_ids_request::RoleIdsRequest;
use crate::wire::dto::admin::user_admin_response_factory::UserAdminResponseFactory;
use crate::wire::dto::admin::user_create_request_factory::UserCreateRequestFactory;
use crate::wire::dto::admin::user_list_response_factory::UserListResponseFactory;
use crate::wire::dto::admin::user_password_reset_request_factory::UserPasswordResetRequestFactory;
use crate::wire::dto::admin::user_update_request_factory::UserUpdateRequestFactory;
use crate::wire::json_body::JsonBody;
use crate::wire::no_content::NoContent;
use crate::wire::wire::Wire;

/// Os handlers de usuário.
pub struct UserHandlers<U> {
    /// O caso de uso de usuário.
    users: U,
}

impl<U: UserUseCase> UserHandlers<U> {
    /// Monta os handlers.
    pub(crate) const fn new(users: U) -> Self {
        Self { users }
    }

    /// `GET /users`
    pub(crate) async fn list(
        &self,
        wire: Wire,
        Query(params): Query<UserPageParams>,
    ) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

        let view = self
            .users
            .list(ListUsersQuery {
                context,
                page: params.page,
                limit: params.limit,
            })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ApiResponse::ok(wire, UserListResponseFactory::of(view)))
    }

    /// `POST /users`
    pub(crate) async fn create(
        &self,
        wire: Wire,
        Body(request): Body<UserCreateRequestFactory>,
    ) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

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
            .map_err(ApiError::of_app)?;

        let user = self.read(context, user.id().to_owned()).await?;

        Ok(ApiResponse::created(wire, user))
    }

    /// `GET /users/{id}`
    pub(crate) async fn get(
        &self,
        wire: Wire,
        Path(id): Path<String>,
    ) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

        Ok(ApiResponse::ok(wire, self.read(context, id).await?))
    }

    /// `PUT /users/{id}`
    pub(crate) async fn update(
        &self,
        wire: Wire,
        Path(id): Path<String>,
        Body(request): Body<UserUpdateRequestFactory>,
    ) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

        let user = self
            .users
            .update(UpdateUserCommand {
                context: context.clone(),
                id,
                name: request.name.unwrap_or_default(),
                email: request.email.unwrap_or_default(),
            })
            .await
            .map_err(ApiError::of_app)?;

        let user = self.read(context, user.id().to_owned()).await?;

        Ok(ApiResponse::ok(wire, user))
    }

    /// `PUT /users/{id}/roles`
    /// Um id vazio na lista é descartado: ele não nomeia papel nenhum, e
    /// deixá-lo passar transformaria um corpo desleixado num 404 confuso.
    pub(crate) async fn update_roles(
        &self,
        wire: Wire,
        Path(id): Path<String>,
        JsonBody(request): JsonBody<RoleIdsRequest>,
    ) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

        let user = self
            .users
            .update_roles(UpdateUserRolesCommand {
                context: context.clone(),
                id,
                role_ids: request
                    .role_ids
                    .into_iter()
                    .filter(|id| !id.is_empty())
                    .collect(),
            })
            .await
            .map_err(ApiError::of_app)?;

        let user = self.read(context, user.id().to_owned()).await?;

        Ok(ApiResponse::ok(wire, user))
    }

    /// `PUT /users/{id}/password`
    pub(crate) async fn reset_password(
        &self,
        Path(id): Path<String>,
        Body(request): Body<UserPasswordResetRequestFactory>,
    ) -> Result<NoContent, ApiError> {
        let context = Session::require_user()?;

        self.users
            .reset_password(ResetUserPasswordCommand {
                context,
                id,
                new_password: request.new_password.unwrap_or_default(),
            })
            .await
            .map_err(ApiError::of_app)?;

        Ok(NoContent::new())
    }

    /// `DELETE /users/{id}`
    pub(crate) async fn delete(&self, Path(id): Path<String>) -> Result<NoContent, ApiError> {
        let context = Session::require_user()?;

        self.users
            .delete(DeleteUserCommand { context, id })
            .await
            .map_err(ApiError::of_app)?;

        Ok(NoContent::new())
    }

    /// O usuário pelo lado de leitura, já na forma do fio.
    async fn read(
        &self,
        context: UserContext,
        id: String,
    ) -> Result<UserAdminResponseFactory, ApiError> {
        let view = self
            .users
            .get(GetUserQuery { context, id })
            .await
            .map_err(ApiError::of_app)?;

        Ok(UserAdminResponseFactory::of(view))
    }
}
