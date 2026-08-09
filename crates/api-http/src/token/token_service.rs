//! O contrato de quem emite e confere o token de sessão.

use portmaster_app::context::UserContext;
use portmaster_app::domain::User;

use crate::error::api_error::ApiError;

/// Emite e confere tokens de sessão.
///
/// Nenhuma outra camada sabe o que é um JWT — e, com esta trait, nem o
/// controller de auth sabe. Ele pede um token e recebe uma string; se um dia a
/// sessão virar cookie de servidor, muda a impl e mais nada.
///
/// É a razão de a trait existir: antes o controller declarava o campo com o tipo
/// concreto, e a hierarquia que as traits desenham valia para fora do crate mas
/// não para dentro dele.
pub(crate) trait TokenService: Clone + Send + Sync + 'static {
    /// O access token de um usuário.
    fn issue(&self, user: &dyn User) -> Result<String, ApiError>;

    /// O principal que o token carrega, se ele valer.
    ///
    /// Recusa por qualquer motivo — assinatura, validade, emissor, payload
    /// ilegível — responde igual: `None`. Quem chama só precisa saber se há
    /// sessão, e distinguir "expirou" de "foi forjado" não muda o que ele faz.
    fn verify(&self, token: &str) -> Option<UserContext>;
}
