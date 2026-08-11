//! O VO de `RoleListResponse`.

use crate::wire::dto::json::admin::role_list_response_json::RoleListResponseJson;
use crate::wire::tables as fbs;
use crate::wire::vo::account::role_x_response::RoleXResponse;
use crate::wire::x::response_x::ResponseX;
use portmaster_app::views::RoleListView;

/// O que a rota de `RoleListResponse` responde.
#[derive(Debug, Clone)]
pub(crate) struct RoleListXResponse {
    /// A página de papéis.
    pub(crate) data: Vec<RoleXResponse>,
    /// Por onde continuar, ou `None` na última página.
    pub(crate) next_cursor: Option<String>,
    /// Quantos papéis existem ao todo.
    pub(crate) total: i32,
}

impl ResponseX for RoleListXResponse {
    type Json = RoleListResponseJson;
    type Fbs = fbs::admin::RoleListResponse;

    fn to_json(&self) -> Self::Json {
        RoleListResponseJson {
            data: self.data.iter().map(ResponseX::to_json).collect(),
            next_cursor: self.next_cursor.clone(),
            total: self.total,
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::admin::RoleListResponse {
            data: Some(self.data.iter().map(ResponseX::to_fbs).collect()),
            next_cursor: self.next_cursor.clone(),
            total: self.total,
        }
    }
}

impl RoleListXResponse {
    /// A página de papéis.
    pub(crate) fn of(source: RoleListView) -> Self {
        Self {
            data: RoleXResponse::of_all(source.items),
            next_cursor: source.next_cursor,
            total: i32::try_from(source.total).unwrap_or(i32::MAX),
        }
    }
}
