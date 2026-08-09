//! O DTO de JSON de `UserCreateRequest`.

use serde::Deserialize;

/// `UserCreateRequest` como o serde o lê.
///
/// Todo campo é opcional: um que falte chega como `None` e o `TableModule`
/// o recusa nomeando-o, em lote com os demais.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct UserCreateRequestJson {
    /// O nome do usuário novo.
    pub(crate) name: Option<String>,
    /// O e-mail, que também é a credencial de login.
    pub(crate) email: Option<String>,
    /// A senha inicial. Morre no `TableModule`, que guarda só o hash.
    pub(crate) initial_password: Option<String>,
    /// Os papéis com que ele nasce.
    pub(crate) role_ids: Option<Vec<String>>,
}
