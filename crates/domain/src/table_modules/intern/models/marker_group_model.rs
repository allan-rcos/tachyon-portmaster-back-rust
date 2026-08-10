//! A implementação de domínio de um grupo de marcador.

use crate::domain::MarkerGroup;

/// A implementação do domínio de [`MarkerGroup`].
pub(crate) struct MarkerGroupModel {
    /// O slug do grupo, que nomeia para que os marcadores dele servem.
    slug: String,
}

impl MarkerGroupModel {
    /// Monta o metadado a partir de um slug já validado.
    pub(crate) const fn new(slug: String) -> Self {
        Self { slug }
    }
}

impl MarkerGroup for MarkerGroupModel {
    fn slug(&self) -> &str {
        &self.slug
    }
}
