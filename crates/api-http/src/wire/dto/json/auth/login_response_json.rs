//! O DTO de JSON de `LoginResponse`.

use crate::wire::dto::json::auth::user_json::UserJson;
use serde::Serialize;

/// `LoginResponse` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct LoginResponseJson {
    /// O access token emitido.
    pub(crate) token: String,
    /// Como o token viaja — `cookie`, e não `Bearer`.
    pub(crate) token_type: String,
    /// O dono da sessão, na forma enxuta que o login publica.
    pub(crate) user: UserJson,
}
