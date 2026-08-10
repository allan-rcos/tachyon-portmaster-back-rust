//! O DTO de JSON de `UserRolesUpdateRequest`.

use serde::Deserialize;

/// O corpo de `PUT /users/{id}/roles` como o serde o lê.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct UserRolesUpdateRequestJson {
    /// O conjunto **final** de papéis; o que ficar de fora é retirado.
    pub(crate) role_ids: Option<Vec<String>>,
}
