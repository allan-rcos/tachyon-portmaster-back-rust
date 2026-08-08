//! Autenticar.

/// Autenticar-se.
///
/// Não carrega contexto: é o comando que **produz** um.
#[derive(Debug, Clone)]
pub struct LoginCommand {
    /// E-mail cadastrado.
    pub email: String,
    /// Senha em claro.
    pub password: String,
}
