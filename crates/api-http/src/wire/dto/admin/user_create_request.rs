//! O que `POST /users` recebe.

/// O que descreve a criação de um usuário.
#[derive(Debug, Clone, Default)]
pub(crate) struct UserCreateRequest {
    /// `name`.
    pub(crate) name: Option<String>,
    /// `email`.
    pub(crate) email: Option<String>,
    /// `initial_password`.
    pub(crate) initial_password: Option<String>,
    /// `role_ids`.
    pub(crate) role_ids: Option<Vec<String>>,
}
