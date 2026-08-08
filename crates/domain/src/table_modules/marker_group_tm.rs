//! As regras de grupo de marcador.

use crate::error::MetadataError;
use crate::models::MarkerGroup;

/// Constrói grupos de marcador.
pub trait MarkerGroupTM {
    /// Cria o grupo de um slug.
    fn create(&self, slug: String) -> Result<Box<dyn MarkerGroup>, MetadataError>;
}
