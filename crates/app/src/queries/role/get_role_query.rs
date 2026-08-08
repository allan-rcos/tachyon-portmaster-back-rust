//! Ler um papel.

use crate::context::UserContext;

/// Ler um papel.
#[derive(Debug, Clone)]
pub struct GetRoleQuery {
    /// Quem está consultando.
    pub context: UserContext,
    /// Id do papel, em base62.
    pub id: String,
}
