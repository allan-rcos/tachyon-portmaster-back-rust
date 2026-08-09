//! O DTO de JSON de `UserAdminPasswordResetRequest`.

use serde::Deserialize;

/// `UserAdminPasswordResetRequest` como o serde o lê.
///
/// Todo campo é opcional: um que falte chega como `None` e o `TableModule`
/// o recusa nomeando-o, em lote com os demais.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct UserAdminPasswordResetRequestJson {
    /// A senha nova, definida por quem administra.
    pub(crate) new_password: Option<String>,
}
