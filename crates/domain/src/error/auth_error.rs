//! O que impede uma autenticação de passar.

/// Falhas de autenticação.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// Senha não confere. A mensagem não distingue e-mail inexistente de senha
    /// errada — dizer qual dos dois falhou entrega ao atacante metade da
    /// resposta.
    #[error("Invalid e-mail or password.")]
    InvalidCredentials,
}
