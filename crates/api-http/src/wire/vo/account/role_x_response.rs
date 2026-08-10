//! O VO de `RoleResponse`.

use crate::wire::convert::Convert;
use crate::wire::dto::json::account::role_response_json::RoleResponseJson;
use crate::wire::tables as fbs;
use crate::wire::x::response_x::ResponseX;
use portmaster_app::views::RoleViewItem;

/// O que a rota de `RoleResponse` responde.
#[derive(Debug, Clone)]
pub(crate) struct RoleXResponse {
    /// Identidade, em base62.
    pub(crate) id: String,
    /// Nome do papel.
    pub(crate) name: String,
    /// Quantos usuários o têm.
    pub(crate) user_count: i32,
    /// Os slugs que ele concede.
    pub(crate) permissions: Vec<String>,
}

impl ResponseX for RoleXResponse {
    type Json = RoleResponseJson;
    type Fbs = fbs::account::RoleResponse;

    fn to_json(&self) -> Self::Json {
        RoleResponseJson {
            id: self.id.clone(),
            name: self.name.clone(),
            user_count: self.user_count,
            permissions: self.permissions.clone(),
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::account::RoleResponse {
            id: Some(self.id.clone()),
            name: Some(self.name.clone()),
            user_count: self.user_count,
            permissions: Some(self.permissions.clone()),
        }
    }
}

impl RoleXResponse {
    /// O papel, vindo do lado de leitura.
    pub(crate) fn of(source: RoleViewItem) -> Self {
        Self {
            id: source.id,
            name: source.name,
            user_count: Convert::count(source.user_count),
            permissions: source.permissions,
        }
    }

    /// A lista de papéis que perfil e usuário-admin publicam igual.
    ///
    /// As duas mensagens carregam o mesmo recorte, e duplicar a conversão nos
    /// dois lugares criaria a chance de uma divergir da outra.
    pub(crate) fn of_all(roles: Vec<RoleViewItem>) -> Vec<Self> {
        roles.into_iter().map(Self::of).collect()
    }
}
