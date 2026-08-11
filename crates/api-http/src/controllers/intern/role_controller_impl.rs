//! O controller de papéis. Não sai do módulo.

use axum::http::StatusCode;
use portmaster_app::commands::role::{CreateRoleCommand, UpdateRolePermissionsCommand};
use portmaster_app::context::UserContext;
use portmaster_app::error::RoleError;
use portmaster_app::queries::role::{GetRoleQuery, ListRolesQuery};
use portmaster_app::services::RoleUseCase;

use crate::controllers::params::page_params::PageParams;
use crate::controllers::role_controller::RoleController;
use crate::ports::error::api_error::ApiError;
use crate::wire::vo::account::role_x_response::RoleXResponse;
use crate::wire::vo::admin::role_create_x_request::RoleCreateXRequest;
use crate::wire::vo::admin::role_list_x_response::RoleListXResponse;
use crate::wire::vo::admin::role_permissions_update_x_request::RolePermissionsUpdateXRequest;

/// Os handlers de papel, genéricos sobre o caso de uso.
#[derive(Clone)]
pub(crate) struct RoleControllerImpl<R> {
    /// O caso de uso de papel.
    roles: R,
}

impl<R: RoleUseCase> RoleControllerImpl<R> {
    /// Monta o controller.
    pub(crate) const fn new(roles: R) -> Self {
        Self { roles }
    }

    /// O papel pelo lado de leitura, já na forma do fio.
    async fn read(&self, context: UserContext, id: String) -> Result<RoleXResponse, ApiError> {
        let view = self
            .roles
            .get(GetRoleQuery { context, id })
            .await
            .map_err(to_api)?;

        Ok(RoleXResponse::of(view))
    }
}

impl<R: RoleUseCase + Clone + Send + Sync + 'static> RoleController for RoleControllerImpl<R> {
    async fn list(
        &self,
        context: UserContext,
        params: PageParams,
    ) -> Result<RoleListXResponse, ApiError> {
        let view = self
            .roles
            .list(ListRolesQuery {
                context,
                cursor: params.cursor,
                limit: params.limit,
                search: params.search,
            })
            .await
            .map_err(to_api)?;

        Ok(RoleListXResponse::of(view))
    }

    /// Cria o papel e o relê pelo lado de leitura.
    ///
    /// A releitura é o que faz a resposta trazer `user_count` — o objeto de
    /// domínio recém-criado não o conhece, e ele faz parte do que a mensagem
    /// publica.
    async fn create(
        &self,
        context: UserContext,
        request: RoleCreateXRequest,
    ) -> Result<RoleXResponse, ApiError> {
        let role = self
            .roles
            .create(CreateRoleCommand {
                context: context.clone(),
                name: request.name.unwrap_or_default(),
                permissions: request.permissions.unwrap_or_default(),
            })
            .await
            .map_err(to_api)?;

        self.read(context, role.id().to_owned()).await
    }

    async fn update_permissions(
        &self,
        context: UserContext,
        id: String,
        request: RolePermissionsUpdateXRequest,
    ) -> Result<RoleXResponse, ApiError> {
        let role = self
            .roles
            .update_permissions(UpdateRolePermissionsCommand {
                context: context.clone(),
                id,
                permissions: request.permissions.unwrap_or_default(),
            })
            .await
            .map_err(to_api)?;

        self.read(context, role.id().to_owned()).await
    }
}

/// Traduz a recusa do serviço de papéis no status que o cliente recebe.
fn to_api(error: RoleError) -> ApiError {
    match error {
        RoleError::Missing(id) => {
            ApiError::new(StatusCode::NOT_FOUND, format!("Role {id} was not found."))
        }
        RoleError::App(shared) => ApiError::of_app(shared),
    }
}
