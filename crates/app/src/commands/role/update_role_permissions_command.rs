//! Trocar as permissões de um papel.

use crate::context::UserContext;

/// Trocar as permissões de um papel.
#[derive(Debug, Clone)]
pub struct UpdateRolePermissionsCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Id do papel, em base62.
    pub id: String,
    /// O conjunto novo — substitui, não soma.
    pub permissions: Vec<String>,
}
