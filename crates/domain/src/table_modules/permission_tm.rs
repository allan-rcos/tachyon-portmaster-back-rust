//! As regras de permissão.

use crate::error::MetadataError;
use crate::models::Permission;

/// Constrói permissões, recusando slug fora do formato.
pub trait PermissionTM {
    /// Cria a permissão de um slug.
    fn create(&self, slug: String) -> Result<Box<dyn Permission>, MetadataError>;
}
