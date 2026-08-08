//! Trocar os papéis de um usuário.

use crate::context::UserContext;

/// Trocar os papéis de um usuário.
#[derive(Debug, Clone)]
pub struct UpdateUserRolesCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Id do usuário, em base62.
    pub id: String,
    /// O conjunto novo — substitui, não soma.
    pub role_ids: Vec<String>,
}
