//! As regras de grupo de marcador.

use crate::domain::MarkerGroup;
use crate::error::MetadataError;

/// Constrói grupos de marcador.
pub trait MarkerGroupTM {
    /// Cria o grupo de um slug.
    fn create(&self, slug: String) -> Result<Box<dyn MarkerGroup>, MetadataError>;
}
