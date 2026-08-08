//! Criar um papel.

use crate::context::UserContext;

/// Criar um papel.
#[derive(Debug, Clone)]
pub struct CreateRoleCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Nome do papel.
    pub name: String,
    /// Os slugs de permissão que ele concede.
    pub permissions: Vec<String>,
}
