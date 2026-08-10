//! O contrato do controller de usuários.

use portmaster_app::context::UserContext;

use crate::controllers::params::user_page_params::UserPageParams;
use crate::error::api_error::ApiError;
use crate::wire::vo::admin::user_admin_password_reset_x_request::UserAdminPasswordResetXRequest;
use crate::wire::vo::admin::user_admin_x_response::UserAdminXResponse;
use crate::wire::vo::admin::user_create_x_request::UserCreateXRequest;
use crate::wire::vo::admin::user_list_x_response::UserListXResponse;
use crate::wire::vo::admin::user_roles_update_x_request::UserRolesUpdateXRequest;
use crate::wire::vo::admin::user_update_x_request::UserUpdateXRequest;

/// Os handlers de usuário.
#[trait_variant::make(Send)]
pub(crate) trait UserController: Clone + Sync + 'static {
    /// `GET /users`
    async fn list(
        &self,
        context: UserContext,
        params: UserPageParams,
    ) -> Result<UserListXResponse, ApiError>;

    /// `POST /users`
    async fn create(
        &self,
        context: UserContext,
        request: UserCreateXRequest,
    ) -> Result<UserAdminXResponse, ApiError>;

    /// `GET /users/{id}`
    async fn get(&self, context: UserContext, id: String) -> Result<UserAdminXResponse, ApiError>;

    /// `PUT /users/{id}`
    async fn update(
        &self,
        context: UserContext,
        id: String,
        request: UserUpdateXRequest,
    ) -> Result<UserAdminXResponse, ApiError>;

    /// `PUT /users/{id}/roles`
    async fn update_roles(
        &self,
        context: UserContext,
        id: String,
        request: UserRolesUpdateXRequest,
    ) -> Result<UserAdminXResponse, ApiError>;

    /// `PUT /users/{id}/password`
    async fn reset_password(
        &self,
        context: UserContext,
        id: String,
        request: UserAdminPasswordResetXRequest,
    ) -> Result<(), ApiError>;

    /// `DELETE /users/{id}`
    async fn delete(&self, context: UserContext, id: String) -> Result<(), ApiError>;
}
