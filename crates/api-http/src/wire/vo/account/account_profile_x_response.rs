//! O VO de `AccountProfileResponse`.

use crate::wire::dto::json::account::account_profile_response_json::AccountProfileResponseJson;
use crate::wire::tables as fbs;
use crate::wire::vo::account::role_x_response::RoleXResponse;
use crate::wire::x::response_x::ResponseX;
use portmaster_app::views::AccountView;

/// O que a rota de `AccountProfileResponse` responde.
#[derive(Debug, Clone)]
pub(crate) struct AccountProfileXResponse {
    /// Identidade, em base62.
    pub(crate) id: String,
    /// Nome de exibição.
    pub(crate) name: String,
    /// E-mail da conta.
    pub(crate) email: String,
    /// Os papéis do dono da conta.
    pub(crate) roles: Vec<RoleXResponse>,
}

impl ResponseX for AccountProfileXResponse {
    type Json = AccountProfileResponseJson;
    type Fbs = fbs::account::AccountProfileResponse;

    fn to_json(&self) -> Self::Json {
        AccountProfileResponseJson {
            id: self.id.clone(),
            name: self.name.clone(),
            email: self.email.clone(),
            roles: self.roles.iter().map(ResponseX::to_json).collect(),
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::account::AccountProfileResponse {
            id: Some(self.id.clone()),
            name: Some(self.name.clone()),
            email: Some(self.email.clone()),
            roles: Some(self.roles.iter().map(ResponseX::to_fbs).collect()),
        }
    }
}

impl AccountProfileXResponse {
    /// O perfil, vindo do lado de leitura.
    pub(crate) fn of(source: AccountView) -> Self {
        Self {
            id: source.id,
            name: source.name,
            email: source.email,
            roles: RoleXResponse::of_all(source.roles),
        }
    }
}
