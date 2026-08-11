//! O contrato do controller de usuários.

use crate::controllers::params::user_page_params::UserPageParams;
use crate::wire::api_response::ApiResponse;
use crate::wire::body::Body;
use crate::wire::vo::admin::user_admin_password_reset_x_request::UserAdminPasswordResetXRequest;
use crate::wire::vo::admin::user_admin_x_response::UserAdminXResponse;
use crate::wire::vo::admin::user_create_x_request::UserCreateXRequest;
use crate::wire::vo::admin::user_list_x_response::UserListXResponse;
use crate::wire::vo::admin::user_roles_update_x_request::UserRolesUpdateXRequest;
use crate::wire::vo::admin::user_update_x_request::UserUpdateXRequest;
use axum::extract::{Path, Query};

/// Os handlers de usuário.
#[trait_variant::make(Send)]
pub(crate) trait UserController: Clone + Sync + 'static {
    /// `GET /users`
    async fn list(self, params: Query<UserPageParams>) -> ApiResponse<UserListXResponse>;

    /// `POST /users`
    async fn create(self, request: Body<UserCreateXRequest>) -> ApiResponse<UserAdminXResponse>;

    /// `GET /users/{id}`
    async fn get(self, id: Path<String>) -> ApiResponse<UserAdminXResponse>;

    /// `PUT /users/{id}`
    async fn update(
        self,
        id: Path<String>,
        request: Body<UserUpdateXRequest>,
    ) -> ApiResponse<UserAdminXResponse>;

    /// `PUT /users/{id}/roles`
    async fn update_roles(
        self,
        id: Path<String>,
        request: Body<UserRolesUpdateXRequest>,
    ) -> ApiResponse<UserAdminXResponse>;

    /// `PUT /users/{id}/password`
    async fn reset_password(
        self,
        id: Path<String>,
        request: Body<UserAdminPasswordResetXRequest>,
    ) -> ApiResponse;

    /// `DELETE /users/{id}`
    async fn delete(self, id: Path<String>) -> ApiResponse;
}
