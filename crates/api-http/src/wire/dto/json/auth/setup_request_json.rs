//! O DTO de JSON de `SetupRequest`.

use serde::Deserialize;

/// `SetupRequest` como o serde o lê.
///
/// Todo campo é opcional: um que falte chega como `None` e o `TableModule`
/// o recusa nomeando-o, em lote com os demais.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct SetupRequestJson {
    /// O nome do primeiro usuário.
    pub(crate) name: Option<String>,
    /// O e-mail dele.
    pub(crate) email: Option<String>,
    /// A senha inicial.
    pub(crate) password: Option<String>,
}
