//! Listar as permissões registradas.

use crate::context::UserContext;

/// Listar as permissões registradas.
#[derive(Debug, Clone)]
pub struct ListPermissionsQuery {
    /// Quem está consultando.
    pub context: UserContext,
    /// Filtra por trecho do slug.
    pub search: Option<String>,
}
