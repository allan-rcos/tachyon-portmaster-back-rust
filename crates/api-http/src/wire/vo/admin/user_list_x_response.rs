//! O VO de `UserListResponse`.

use crate::wire::dto::json::admin::user_list_response_json::UserListResponseJson;
use crate::wire::tables as fbs;
use crate::wire::vo::admin::user_admin_x_response::UserAdminXResponse;
use crate::wire::x::response_x::ResponseX;
use portmaster_app::views::UserListView;

/// O que a rota de `UserListResponse` responde.
#[derive(Debug, Clone)]
pub(crate) struct UserListXResponse {
    /// A página de usuários.
    pub(crate) data: Vec<UserAdminXResponse>,
}

impl ResponseX for UserListXResponse {
    type Json = UserListResponseJson;
    type Fbs = fbs::admin::UserListResponse;

    fn to_json(&self) -> Self::Json {
        UserListResponseJson {
            data: self.data.iter().map(ResponseX::to_json).collect(),
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::admin::UserListResponse {
            data: Some(self.data.iter().map(ResponseX::to_fbs).collect()),
        }
    }
}

impl UserListXResponse {
    /// A página de usuários.
    pub(crate) fn of(source: UserListView) -> Self {
        Self {
            data: source.items.into_iter().map(UserAdminXResponse::of).collect(),
        }
    }
}
