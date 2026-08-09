//! O VO de `User`.

use crate::wire::dto::json::auth::user_json::UserJson;
use crate::wire::tables as fbs;
use crate::wire::x::response_x::ResponseX;

/// O que a rota de `User` responde.
#[derive(Debug, Clone)]
pub(crate) struct UserX {
    /// Identidade, em base62.
    pub(crate) id: String,
    /// Nome de exibição.
    pub(crate) name: String,
    /// E-mail do dono da sessão.
    pub(crate) email: String,
}

impl ResponseX for UserX {
    type Json = UserJson;
    type Fbs = fbs::auth::User;

    fn to_json(&self) -> Self::Json {
        UserJson {
            id: self.id.clone(),
            name: self.name.clone(),
            email: self.email.clone(),
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::auth::User {
            id: Some(self.id.clone()),
            name: Some(self.name.clone()),
            email: Some(self.email.clone()),
        }
    }
}

impl UserX {
    /// O usuário, vindo do objeto de domínio.
    ///
    /// O que **não** atravessa é a garantia: um `User` de domínio tem
    /// `password_hash`, e este VO não tem onde pôr isso.
    pub(crate) fn of(user: &dyn portmaster_app::domain::User) -> Self {
        Self {
            id: user.id().to_owned(),
            name: user.name().to_owned(),
            email: user.email().to_owned(),
        }
    }
}
