//! O DTO de JSON de `RoleCreateRequest`.

use serde::Deserialize;

/// `RoleCreateRequest` como o serde o lê.
///
/// Todo campo é opcional: um que falte chega como `None` e o `TableModule`
/// o recusa nomeando-o, em lote com os demais.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct RoleCreateRequestJson {
    /// O nome do papel novo.
    pub(crate) name: Option<String>,
    /// Os slugs que ele concede.
    pub(crate) permissions: Option<Vec<String>>,
}
