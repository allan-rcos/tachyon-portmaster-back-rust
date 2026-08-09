//! O DTO de JSON de `RoleResponse`.

use serde::Serialize;

/// `RoleResponse` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct RoleResponseJson {
    /// Identidade, em base62.
    pub(crate) id: String,
    /// Nome do papel.
    pub(crate) name: String,
    /// Quantos usuários o têm.
    pub(crate) user_count: i32,
    /// Os slugs que ele concede.
    pub(crate) permissions: Vec<String>,
}
