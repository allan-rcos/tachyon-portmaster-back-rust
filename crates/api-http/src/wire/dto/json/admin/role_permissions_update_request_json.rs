//! O DTO de JSON de `RolePermissionsUpdateRequest`.

use serde::Deserialize;

/// `RolePermissionsUpdateRequest` como o serde o lê.
///
/// Todo campo é opcional: um que falte chega como `None` e o `TableModule`
/// o recusa nomeando-o, em lote com os demais.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct RolePermissionsUpdateRequestJson {
    /// O conjunto **final** de permissões; o que ficar de fora é retirado.
    pub(crate) permissions: Option<Vec<String>>,
}
