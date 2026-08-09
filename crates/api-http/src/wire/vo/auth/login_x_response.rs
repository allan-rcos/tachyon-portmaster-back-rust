//! O VO de `LoginResponse`.

use crate::wire::dto::json::auth::login_response_json::LoginResponseJson;
use crate::wire::tables as fbs;
use crate::wire::vo::auth::user_x::UserX;
use crate::wire::x::response_x::ResponseX;

/// O que a rota de `LoginResponse` responde.
#[derive(Debug, Clone)]
pub(crate) struct LoginXResponse {
    /// O access token emitido.
    pub(crate) token: String,
    /// Como o token viaja — `cookie`, e não `Bearer`.
    pub(crate) token_type: String,
    /// O dono da sessão, na forma enxuta que o login publica.
    pub(crate) user: UserX,
}

impl ResponseX for LoginXResponse {
    type Json = LoginResponseJson;
    type Fbs = fbs::auth::LoginResponse;

    fn to_json(&self) -> Self::Json {
        LoginResponseJson {
            token: self.token.clone(),
            token_type: self.token_type.clone(),
            user: self.user.to_json(),
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::auth::LoginResponse {
            token: Some(self.token.clone()),
            token_type: Some(self.token_type.clone()),
            user: Some(Box::new(self.user.to_fbs())),
        }
    }
}
