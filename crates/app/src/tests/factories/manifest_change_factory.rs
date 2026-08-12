//! A mudança de manifesto que o domínio descreveria, montada para o teste.

use portmaster_domain::domain::{Container, ManifestCargo, ManifestChange};
use portmaster_domain::enums::{ContainerStatus, TelemetryEvent};

use crate::tests::factories::container_factory::StubContainer;

/// Uma mudança que o teste controla nos três ramos que ela decide.
///
/// Os ramos são exclusivos e vêm daqui, e não do service: limpar o manifesto,
/// apagar a linha, ou gravá-la. É o que permite afirmar que o service obedece à
/// decisão do domínio em vez de tomá-la de novo.
pub(crate) struct StubManifestChange {
    /// O contêiner já movido.
    container: Box<dyn Container>,
    /// O produto que se moveu.
    product_id: String,
    /// A linha resultante, quando há uma.
    cargo: Option<Box<dyn ManifestCargo>>,
    /// Se o manifesto inteiro deve ser limpo.
    clear_manifest: bool,
}

impl StubManifestChange {
    /// A mudança que manda **limpar o manifesto** inteiro.
    pub(crate) fn clearing(container_id: &str, product_id: &str) -> Box<dyn ManifestChange> {
        Box::new(Self {
            container: StubContainer::boxed(container_id, ContainerStatus::Empty),
            product_id: product_id.to_owned(),
            cargo: None,
            clear_manifest: true,
        })
    }

    /// A mudança que manda **apagar a linha** daquele produto.
    pub(crate) fn removing(container_id: &str, product_id: &str) -> Box<dyn ManifestChange> {
        Box::new(Self {
            container: StubContainer::boxed(container_id, ContainerStatus::Loading),
            product_id: product_id.to_owned(),
            cargo: None,
            clear_manifest: false,
        })
    }

    /// A mudança que manda **gravar a linha** que veio junto.
    pub(crate) fn upserting(
        container_id: &str,
        product_id: &str,
        cargo: Box<dyn ManifestCargo>,
    ) -> Box<dyn ManifestChange> {
        Box::new(Self {
            container: StubContainer::boxed(container_id, ContainerStatus::Loading),
            product_id: product_id.to_owned(),
            cargo: Some(cargo),
            clear_manifest: false,
        })
    }
}

impl ManifestChange for StubManifestChange {
    fn container(&self) -> &dyn Container {
        self.container.as_ref()
    }

    fn product_id(&self) -> &str {
        &self.product_id
    }

    fn cargo(&self) -> Option<&dyn ManifestCargo> {
        self.cargo.as_deref()
    }

    fn clear_manifest(&self) -> bool {
        self.clear_manifest
    }

    fn event(&self) -> TelemetryEvent {
        TelemetryEvent::Load
    }

    fn into_container(self: Box<Self>) -> Box<dyn Container> {
        self.container
    }
}
