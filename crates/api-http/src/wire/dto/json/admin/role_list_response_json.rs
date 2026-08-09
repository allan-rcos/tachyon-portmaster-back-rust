//! O DTO de JSON de `RoleListResponse`.

use crate::wire::dto::json::account::role_response_json::RoleResponseJson;
use serde::Serialize;

/// `RoleListResponse` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct RoleListResponseJson {
    /// A página de papéis.
    pub(crate) data: Vec<RoleResponseJson>,
    /// Por onde continuar, ou `None` na última página.
    pub(crate) next_cursor: Option<String>,
    /// Quantos papéis existem ao todo.
    pub(crate) total: i32,
}
