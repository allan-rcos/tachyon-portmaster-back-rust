//! O DTO de JSON de `User`.

use serde::Serialize;

/// `User` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct UserJson {
    /// Identidade, em base62.
    pub(crate) id: String,
    /// Nome de exibição.
    pub(crate) name: String,
    /// E-mail do dono da sessão.
    pub(crate) email: String,
}
