//! O que `PUT /users/{id}/password` recebe.

/// O que descreve a redefinição de senha de um usuário.
#[derive(Debug, Clone, Default)]
pub(crate) struct UserPasswordResetRequest {
    /// `new_password`.
    pub(crate) new_password: Option<String>,
}
