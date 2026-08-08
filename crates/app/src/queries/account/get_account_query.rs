//! Ler o próprio perfil.

use crate::context::UserContext;

/// Ler a própria conta.
#[derive(Debug, Clone)]
pub struct GetAccountQuery {
    /// Quem está consultando — e de quem é a conta.
    pub context: UserContext,
}
