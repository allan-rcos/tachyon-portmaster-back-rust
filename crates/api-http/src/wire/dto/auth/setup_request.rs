//! O que `POST /setup` recebe.

/// Os dados do primeiro usuário do sistema.
#[derive(Debug, Clone, Default)]
pub(crate) struct SetupRequest {
    /// Nome de exibição.
    pub(crate) name: Option<String>,
    /// E-mail de login.
    pub(crate) email: Option<String>,
    /// Senha em claro.
    pub(crate) password: Option<String>,
}
