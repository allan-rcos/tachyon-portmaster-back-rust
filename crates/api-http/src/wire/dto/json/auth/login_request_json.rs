//! O DTO de JSON de `LoginRequest`.

use serde::Deserialize;

/// `LoginRequest` como o serde o lê.
///
/// Todo campo é opcional: um que falte chega como `None` e o `TableModule`
/// o recusa nomeando-o, em lote com os demais.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct LoginRequestJson {
    /// O e-mail informado.
    pub(crate) email: Option<String>,
    /// A senha em claro. Morre no `TableModule`, que guarda só o hash.
    pub(crate) password: Option<String>,
}
