//! Ler um usuário.

use crate::context::UserContext;

/// Ler um usuário.
#[derive(Debug, Clone)]
pub struct GetUserQuery {
    /// Quem está consultando.
    pub context: UserContext,
    /// Id do usuário, em base62.
    pub id: String,
}
