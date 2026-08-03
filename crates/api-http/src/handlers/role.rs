//! `/roles` — os papéis e o que cada um concede.
//!
//! Não há `GET /roles/{id}` no contrato: um papel é lido pela listagem, que é
//! curta por natureza — o número de papéis é decisão de administração, não
//! volume de dados. A leitura individual existe internamente, para montar a
//! resposta das escritas.
//!
//! ## As permissões são substituídas por inteiro
//!
//! `PUT /roles/{id}/permissions` recebe o conjunto final, não um delta. É o que
//! torna a operação idempotente e o que evita a pergunta "esta lista soma ou
//! substitui?" — a resposta de um `PUT` é sempre substituir.
//!
//! ## Por que a escrita relê antes de responder
//!
//! `RoleResponse` publica `user_count`, que é consulta e não regra: o objeto de
//! domínio devolvido pela escrita não o conhece. Reler pelo lado de leitura é o
//! que faz a resposta da criação ser igual à linha que a listagem vai mostrar.

use axum::extract::{Path, Query};
use portmaster_app::context::UserContext;
use portmaster_app::role::{
    CreateRoleCommand, GetRoleQuery, ListRolesQuery, RoleUseCase, UpdateRolePermissionsCommand,
};

use super::PageParams;
use crate::error::{app_error_to_status, ApiError};
use crate::session::Session;
use crate::wire::http::{Accept, Body, Negotiated};
use crate::wire::tables as fbs;

/// Os handlers de papel.
pub(crate) struct RoleHandlers<R> {
    roles: R,
}

impl<R: RoleUseCase> RoleHandlers<R> {
    /// Monta os handlers.
    pub(crate) fn new(roles: R) -> Self {
        Self { roles }
    }

    /// `GET /roles`
    pub(crate) async fn list(
        &self,
        accept: Accept,
        Query(params): Query<PageParams>,
    ) -> Result<Negotiated<fbs::admin::RoleListResponse>, ApiError> {
        let context = Session::require_user()?;

        let view = self
            .roles
            .list(ListRolesQuery {
                context,
                cursor: params.cursor,
                limit: params.limit,
                search: params.search,
            })
            .await
            .map_err(app_error_to_status)?;

        Ok(Negotiated::ok(accept, view.into()))
    }

    /// `POST /roles`
    pub(crate) async fn create(
        &self,
        accept: Accept,
        Body(request): Body<fbs::admin::RoleCreateRequest>,
    ) -> Result<Negotiated<fbs::account::RoleResponse>, ApiError> {
        let context = Session::require_user()?;

        let role = self
            .roles
            .create(CreateRoleCommand {
                context: context.clone(),
                name: request.name,
                permissions: request.permissions.unwrap_or_default(),
            })
            .await
            .map_err(app_error_to_status)?;

        let view = self.read(context, role.id().to_owned()).await?;

        Ok(Negotiated::created(accept, view))
    }

    /// `PUT /roles/{id}/permissions`
    pub(crate) async fn update_permissions(
        &self,
        accept: Accept,
        Path(id): Path<String>,
        Body(request): Body<fbs::admin::RolePermissionsUpdateRequest>,
    ) -> Result<Negotiated<fbs::account::RoleResponse>, ApiError> {
        let context = Session::require_user()?;

        let role = self
            .roles
            .update_permissions(UpdateRolePermissionsCommand {
                context: context.clone(),
                id,
                permissions: request.permissions.unwrap_or_default(),
            })
            .await
            .map_err(app_error_to_status)?;

        let view = self.read(context, role.id().to_owned()).await?;

        Ok(Negotiated::ok(accept, view))
    }

    /// O papel pelo lado de leitura, já na forma do fio.
    async fn read(
        &self,
        context: UserContext,
        id: String,
    ) -> Result<fbs::account::RoleResponse, ApiError> {
        let view = self
            .roles
            .get(GetRoleQuery { context, id })
            .await
            .map_err(app_error_to_status)?;

        Ok(view.into())
    }
}
