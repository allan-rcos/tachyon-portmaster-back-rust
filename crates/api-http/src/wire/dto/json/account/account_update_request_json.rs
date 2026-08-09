//! O DTO de JSON de `AccountUpdateRequest`.

use serde::Deserialize;

/// `AccountUpdateRequest` como o serde o lê.
///
/// Todo campo é opcional: um que falte chega como `None` e o `TableModule`
/// o recusa nomeando-o, em lote com os demais.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct AccountUpdateRequestJson {
    /// O nome novo.
    pub(crate) name: Option<String>,
    /// O e-mail novo.
    pub(crate) email: Option<String>,
}
