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
use portmaster_app::context::UserContext;
use portmaster_app::user::{
    CreateUserCommand, DeleteUserCommand, GetUserQuery, ListUsersQuery, ResetUserPasswordCommand,
    UpdateUserCommand, UpdateUserRolesCommand, UserUseCase,
};
use serde::Deserialize;

use super::UserPageParams;
use crate::error::{app_error_to_status, ApiError};
use crate::session::Session;
use crate::wire::http::{Accept, Body, JsonBody, Negotiated, NoContent};
use crate::wire::tables as fbs;

/// O corpo de `PUT /users/{id}/roles`.
///
/// Não é uma tabela de `.fbs`: este payload nunca entrou no schema publicado, e
/// inventá-lo agora mudaria o contrato de um endpoint que já está em uso. Ver
/// [`JsonBody`].
#[derive(Debug, Default, Deserialize)]
pub(crate) struct RoleIdsRequest {
    /// O conjunto **final** de papéis; o que ficar de fora é retirado.
    #[serde(default)]
    pub(crate) role_ids: Vec<String>,
}

/// Os handlers de usuário.
pub(crate) struct UserHandlers<U> {
    users: U,
}

impl<U: UserUseCase> UserHandlers<U> {
    /// Monta os handlers.
    pub(crate) fn new(users: U) -> Self {
        Self { users }
    }

    /// `GET /users`
    pub(crate) async fn list(
        &self,
        accept: Accept,
        Query(params): Query<UserPageParams>,
    ) -> Result<Negotiated<fbs::admin::UserListResponse>, ApiError> {
        let context = Session::require_user()?;

        let view = self
            .users
            .list(ListUsersQuery {
                context,
                page: params.page,
                limit: params.limit,
            })
            .await
            .map_err(app_error_to_status)?;

        Ok(Negotiated::ok(accept, view.into()))
    }

    /// `POST /users`
    pub(crate) async fn create(
        &self,
        accept: Accept,
        Body(request): Body<fbs::admin::UserCreateRequest>,
    ) -> Result<Negotiated<fbs::admin::UserAdminResponse>, ApiError> {
        let context = Session::require_user()?;

        let user = self
            .users
            .create(CreateUserCommand {
                context: context.clone(),
                name: request.name,
                email: request.email,
                initial_password: request.initial_password,
                role_ids: request.role_ids.unwrap_or_default(),
            })
            .await
            .map_err(app_error_to_status)?;

        let view = self.read(context, user.id().to_owned()).await?;

        Ok(Negotiated::created(accept, view))
    }

    /// `GET /users/{id}`
    pub(crate) async fn get(
        &self,
        accept: Accept,
        Path(id): Path<String>,
    ) -> Result<Negotiated<fbs::admin::UserAdminResponse>, ApiError> {
        let context = Session::require_user()?;

        Ok(Negotiated::ok(accept, self.read(context, id).await?))
    }

    /// `PUT /users/{id}`
    pub(crate) async fn update(
        &self,
        accept: Accept,
        Path(id): Path<String>,
        Body(request): Body<fbs::admin::UserUpdateRequest>,
    ) -> Result<Negotiated<fbs::admin::UserAdminResponse>, ApiError> {
        let context = Session::require_user()?;

        let user = self
            .users
            .update(UpdateUserCommand {
                context: context.clone(),
                id,
                name: request.name,
                email: request.email,
            })
            .await
            .map_err(app_error_to_status)?;

        let view = self.read(context, user.id().to_owned()).await?;

        Ok(Negotiated::ok(accept, view))
    }

    /// `PUT /users/{id}/roles`
    pub(crate) async fn update_roles(
        &self,
        accept: Accept,
        Path(id): Path<String>,
        JsonBody(request): JsonBody<RoleIdsRequest>,
    ) -> Result<Negotiated<fbs::admin::UserAdminResponse>, ApiError> {
        let context = Session::require_user()?;

        let user = self
            .users
            .update_roles(UpdateUserRolesCommand {
                context: context.clone(),
                id,
                // Um id vazio na lista não nomeia papel nenhum, e deixá-lo
                // passar transformaria um corpo desleixado num 404 confuso.
                role_ids: request
                    .role_ids
                    .into_iter()
                    .filter(|id| !id.is_empty())
                    .collect(),
            })
            .await
            .map_err(app_error_to_status)?;

        let view = self.read(context, user.id().to_owned()).await?;

        Ok(Negotiated::ok(accept, view))
    }

    /// `PUT /users/{id}/password`
    pub(crate) async fn reset_password(
        &self,
        Path(id): Path<String>,
        Body(request): Body<fbs::admin::UserAdminPasswordResetRequest>,
    ) -> Result<NoContent, ApiError> {
        let context = Session::require_user()?;

        self.users
            .reset_password(ResetUserPasswordCommand {
                context,
                id,
                new_password: request.new_password,
            })
            .await
            .map_err(app_error_to_status)?;

        Ok(NoContent::new())
    }

    /// `DELETE /users/{id}`
    pub(crate) async fn delete(&self, Path(id): Path<String>) -> Result<NoContent, ApiError> {
        let context = Session::require_user()?;

        self.users
            .delete(DeleteUserCommand { context, id })
            .await
            .map_err(app_error_to_status)?;

        Ok(NoContent::new())
    }

    /// O usuário pelo lado de leitura, já na forma do fio.
    async fn read(
        &self,
        context: UserContext,
        id: String,
    ) -> Result<fbs::admin::UserAdminResponse, ApiError> {
        let view = self
            .users
            .get(GetUserQuery { context, id })
            .await
            .map_err(app_error_to_status)?;

        Ok(view.into())
    }
}
