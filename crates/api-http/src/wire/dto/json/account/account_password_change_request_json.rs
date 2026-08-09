//! O DTO de JSON de `AccountPasswordChangeRequest`.

use serde::Deserialize;

/// `AccountPasswordChangeRequest` como o serde o lê.
///
/// Todo campo é opcional: um que falte chega como `None` e o `TableModule`
/// o recusa nomeando-o, em lote com os demais.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct AccountPasswordChangeRequestJson {
    /// A senha atual, que prova ser o dono da conta.
    pub(crate) current_password: Option<String>,
    /// A senha nova.
    pub(crate) new_password: Option<String>,
}
