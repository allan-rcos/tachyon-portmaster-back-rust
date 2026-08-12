//! Um marcador de domínio, montado para o teste.

use portmaster_domain::domain::Marker;

/// Um marcador que o teste controla.
pub(crate) struct StubMarker {
    /// O grupo a que pertence.
    group: String,
    /// A chave já derivada do valor em claro.
    key: String,
}

impl StubMarker {
    /// O marcador deste grupo e chave, dentro do `Box` do table module.
    pub(crate) fn boxed(group: &str, key: &str) -> Box<dyn Marker> {
        Box::new(Self {
            group: group.to_owned(),
            key: key.to_owned(),
        })
    }
}

impl Marker for StubMarker {
    fn group(&self) -> &str {
        &self.group
    }

    fn key(&self) -> &str {
        &self.key
    }

    fn flag(&self) -> bool {
        false
    }
}
