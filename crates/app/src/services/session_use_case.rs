//! Login, validação de sessão e o setup inicial.

use crate::commands::session::LoginCommand;
use crate::commands::session::SetupCommand;
use crate::context::UserContext;
use crate::error::SessionError;
use portmaster_domain::domain::User;

/// O que a apresentação pode pedir sobre sessão.
#[trait_variant::make(Send)]
pub trait SessionUseCase {
    /// Confere as credenciais e devolve o usuário com os papéis dele.
    async fn login(&self, command: LoginCommand) -> Result<Box<dyn User>, SessionError>;

    /// Reconfere que a sessão ainda descreve alguém.
    ///
    /// O token é auto-contido e válido até expirar, então ele continua sendo
    /// aceito depois de o usuário ser removido ou ter os papéis trocados. Isto é
    /// o que confronta a afirmação do token com o banco, para quem quiser pagar
    /// a consulta.
    async fn validate(&self, context: &UserContext) -> Result<Box<dyn User>, SessionError>;

    /// Cria o primeiro usuário e o papel que concede tudo.
    async fn setup(&self, command: SetupCommand) -> Result<Box<dyn User>, SessionError>;
}
