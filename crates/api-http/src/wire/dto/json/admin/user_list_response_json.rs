//! O DTO de JSON de `UserListResponse`.

use crate::wire::dto::json::admin::user_admin_response_json::UserAdminResponseJson;
use serde::Serialize;

/// `UserListResponse` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct UserListResponseJson {
    /// A página de usuários.
    pub(crate) data: Vec<UserAdminResponseJson>,
}
