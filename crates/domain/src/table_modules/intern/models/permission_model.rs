//! A implementação de domínio de uma permissão.

use crate::domain::Permission;

/// A implementação do domínio de [`Permission`].
pub(crate) struct PermissionModel {
    /// O slug da permissão, no formato `recurso:ação`.
    slug: String,
}

impl PermissionModel {
    /// Monta o metadado a partir de um slug já validado.
    pub(crate) const fn new(slug: String) -> Self {
        Self { slug }
    }
}

impl Permission for PermissionModel {
    fn slug(&self) -> &str {
        &self.slug
    }
}
