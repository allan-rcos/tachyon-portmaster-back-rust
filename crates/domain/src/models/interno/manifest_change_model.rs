//! A implementação de domínio do efeito de um movimento de carga.

use crate::enums::TelemetryEvent;
use crate::models::{Container, ManifestCargo, ManifestChange};

/// A implementação do domínio de [`ManifestChange`].
pub struct ManifestChangeModel {
    /// O contêiner **depois** do movimento, com peso e status já recalculados.
    container: Box<dyn Container>,
    /// O produto movimentado, em base62.
    product_id: String,
    /// A linha de manifesto resultante, ou `None` quando o movimento a zerou.
    cargo: Option<Box<dyn ManifestCargo>>,
    /// Se a linha deve sair do manifesto em vez de ser atualizada.
    ///
    /// É o desembarque que levou a quantidade a zero: manter a linha com
    /// zero faria o manifesto crescer com registros que não são carga.
    clear_manifest: bool,
    /// O evento a registrar na telemetria — embarque ou desembarque.
    event: TelemetryEvent,
}

impl ManifestChangeModel {
    /// Monta o efeito de um movimento de carga.
    pub(crate) fn new(
        container: Box<dyn Container>,
        product_id: String,
        cargo: Option<Box<dyn ManifestCargo>>,
        clear_manifest: bool,
        event: TelemetryEvent,
    ) -> Self {
        Self {
            container,
            product_id,
            cargo,
            clear_manifest,
            event,
        }
    }
}

impl ManifestChange for ManifestChangeModel {
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
        self.event
    }

    fn into_container(self: Box<Self>) -> Box<dyn Container> {
        self.container
    }
}
