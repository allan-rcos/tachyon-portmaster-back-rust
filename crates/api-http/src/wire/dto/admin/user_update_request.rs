//! O que `PUT /users/{id}` recebe.

/// O que descreve a alteração de um usuário.
#[derive(Debug, Clone, Default)]
pub(crate) struct UserUpdateRequest {
    /// `name`.
    pub(crate) name: Option<String>,
    /// `email`.
    pub(crate) email: Option<String>,
}
