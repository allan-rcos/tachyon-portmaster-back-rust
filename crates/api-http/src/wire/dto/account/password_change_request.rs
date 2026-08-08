//! O que `PUT /account/password` recebe.

/// O que descreve a troca da própria senha.
#[derive(Debug, Clone, Default)]
pub(crate) struct PasswordChangeRequest {
    /// `current_password`.
    pub(crate) current_password: Option<String>,
    /// `new_password`.
    pub(crate) new_password: Option<String>,
}
