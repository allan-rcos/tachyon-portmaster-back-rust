//! O DTO de JSON de `UserAdminResponse`.

use crate::wire::dto::json::account::role_response_json::RoleResponseJson;
use serde::Serialize;

/// `UserAdminResponse` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct UserAdminResponseJson {
    /// Identidade, em base62.
    pub(crate) id: String,
    /// Nome de exibição.
    pub(crate) name: String,
    /// E-mail do usuário.
    pub(crate) email: String,
    /// Os papéis dele.
    pub(crate) roles: Vec<RoleResponseJson>,
}
