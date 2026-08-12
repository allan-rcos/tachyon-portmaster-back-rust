//! Um grupo de marcador, montado para o teste.

use portmaster_domain::domain::MarkerGroup;

/// Um grupo que o teste controla.
pub(crate) struct StubMarkerGroup {
    /// O slug do grupo.
    slug: String,
}

impl StubMarkerGroup {
    /// O grupo deste slug, dentro do `Box` que o table module devolveria.
    pub(crate) fn boxed(slug: &str) -> Box<dyn MarkerGroup> {
        Box::new(Self {
            slug: slug.to_owned(),
        })
    }
}

impl MarkerGroup for StubMarkerGroup {
    fn slug(&self) -> &str {
        &self.slug
    }
}
