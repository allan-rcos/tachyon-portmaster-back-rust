//! O DTO de JSON de `AccountProfileResponse`.

use crate::wire::dto::json::account::role_response_json::RoleResponseJson;
use serde::Serialize;

/// `AccountProfileResponse` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct AccountProfileResponseJson {
    /// Identidade, em base62.
    pub(crate) id: String,
    /// Nome de exibição.
    pub(crate) name: String,
    /// E-mail da conta.
    pub(crate) email: String,
    /// Os papéis do dono da conta.
    pub(crate) roles: Vec<RoleResponseJson>,
}
