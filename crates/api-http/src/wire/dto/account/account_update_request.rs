//! O que `PUT /account` recebe.

/// O que descreve a alteração do próprio perfil.
#[derive(Debug, Clone, Default)]
pub(crate) struct AccountUpdateRequest {
    /// `name`.
    pub(crate) name: Option<String>,
    /// `email`.
    pub(crate) email: Option<String>,
}
