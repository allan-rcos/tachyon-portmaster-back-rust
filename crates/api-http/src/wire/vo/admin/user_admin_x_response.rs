//! O VO de `UserAdminResponse`.

use crate::wire::dto::json::admin::user_admin_response_json::UserAdminResponseJson;
use crate::wire::tables as fbs;
use crate::wire::vo::account::role_x_response::RoleXResponse;
use crate::wire::x::response_x::ResponseX;
use portmaster_app::views::AccountView;

/// O que a rota de `UserAdminResponse` responde.
#[derive(Debug, Clone)]
pub(crate) struct UserAdminXResponse {
    /// Identidade, em base62.
    pub(crate) id: String,
    /// Nome de exibição.
    pub(crate) name: String,
    /// E-mail do usuário.
    pub(crate) email: String,
    /// Os papéis dele.
    pub(crate) roles: Vec<RoleXResponse>,
}

impl ResponseX for UserAdminXResponse {
    type Json = UserAdminResponseJson;
    type Fbs = fbs::admin::UserAdminResponse;

    fn to_json(&self) -> Self::Json {
        UserAdminResponseJson {
            id: self.id.clone(),
            name: self.name.clone(),
            email: self.email.clone(),
            roles: self.roles.iter().map(ResponseX::to_json).collect(),
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::admin::UserAdminResponse {
            id: Some(self.id.clone()),
            name: Some(self.name.clone()),
            email: Some(self.email.clone()),
            roles: Some(self.roles.iter().map(ResponseX::to_fbs).collect()),
        }
    }
}

impl UserAdminXResponse {
    /// O usuário, vindo do lado de leitura.
    pub(crate) fn of(source: AccountView) -> Self {
        Self {
            id: source.id,
            name: source.name,
            email: source.email,
            roles: RoleXResponse::of_all(source.roles),
        }
    }
}
