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
use portmaster_app::commands::role::CreateRoleCommand;
use portmaster_app::commands::role::UpdateRolePermissionsCommand;
use portmaster_app::context::UserContext;
use portmaster_app::queries::role::GetRoleQuery;
use portmaster_app::queries::role::ListRolesQuery;
use portmaster_app::services::RoleUseCase;

use crate::error::api_error::ApiError;
use crate::handlers::params::page_params::PageParams;
use crate::session::Session;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::dto::account::role_response_factory::RoleResponseFactory;
use crate::wire::dto::admin::role_create_request_factory::RoleCreateRequestFactory;
use crate::wire::dto::admin::role_list_response_factory::RoleListResponseFactory;
use crate::wire::dto::admin::role_permissions_update_request_factory::RolePermissionsUpdateRequestFactory;
use crate::wire::wire::Wire;

/// Os handlers de papel.
pub struct RoleHandlers<R> {
    roles: R,
}

impl<R: RoleUseCase> RoleHandlers<R> {
    /// Monta os handlers.
    pub(crate) const fn new(roles: R) -> Self {
        Self { roles }
    }

    /// `GET /roles`
    pub(crate) async fn list(
        &self,
        wire: Wire,
        Query(params): Query<PageParams>,
    ) -> Result<ApiResponse, ApiError> {
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
            .map_err(ApiError::of_app)?;

        Ok(ApiResponse::ok(wire, RoleListResponseFactory::of(view)))
    }

    /// `POST /roles`
    pub(crate) async fn create(
        &self,
        wire: Wire,
        Body(request): Body<RoleCreateRequestFactory>,
    ) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

        let role = self
            .roles
            .create(CreateRoleCommand {
                context: context.clone(),
                name: request.name.unwrap_or_default(),
                permissions: request.permissions.unwrap_or_default(),
            })
            .await
            .map_err(ApiError::of_app)?;

        let role = self.read(context, role.id().to_owned()).await?;

        Ok(ApiResponse::created(wire, role))
    }

    /// `PUT /roles/{id}/permissions`
    pub(crate) async fn update_permissions(
        &self,
        wire: Wire,
        Path(id): Path<String>,
        Body(request): Body<RolePermissionsUpdateRequestFactory>,
    ) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

        let role = self
            .roles
            .update_permissions(UpdateRolePermissionsCommand {
                context: context.clone(),
                id,
                permissions: request.permissions.unwrap_or_default(),
            })
            .await
            .map_err(ApiError::of_app)?;

        let role = self.read(context, role.id().to_owned()).await?;

        Ok(ApiResponse::ok(wire, role))
    }

    /// O papel pelo lado de leitura, já na forma do fio.
    async fn read(
        &self,
        context: UserContext,
        id: String,
    ) -> Result<RoleResponseFactory, ApiError> {
        let view = self
            .roles
            .get(GetRoleQuery { context, id })
            .await
            .map_err(ApiError::of_app)?;

        Ok(RoleResponseFactory::of(view))
    }
}
