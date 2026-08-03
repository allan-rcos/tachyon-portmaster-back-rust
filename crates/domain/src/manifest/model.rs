//! A carga de um contêiner, e o efeito de embarcar ou desembarcar.

use chrono::{DateTime, Utc};

use crate::container::Container;
use crate::enums::TelemetryEvent;

/// Uma linha do manifesto: quanto de um produto está num contêiner.
///
/// É uma entidade **fraca** — satélite do contêiner, sem sentido sozinha. Por
/// isso carrega só `created_at`: não é atualizável nem sofre soft-delete, e
/// mudar uma linha é removê-la e recriá-la.
pub trait ManifestCargo: Send + Sync {
    /// Contêiner que carrega o item, em base62.
    fn container_id(&self) -> &str;

    /// Produto embarcado, em base62.
    fn product_id(&self) -> &str;

    /// Quantidade embarcada.
    fn quantity(&self) -> f64;

    /// Peso correspondente, em quilos.
    fn weight(&self) -> f64;

    /// Quando a linha nasceu.
    fn created_at(&self) -> DateTime<Utc>;
}

/// O efeito completo de um embarque ou desembarque.
///
/// Um único movimento de carga toca três coisas — o peso e o status do
/// contêiner, a linha do manifesto, e o registro de telemetria — e as três
/// precisam ser gravadas juntas ou nenhuma. Devolvê-las num objeto só é o que
/// impede o `app` de aplicar metade.
pub trait ManifestChange: Send + Sync {
    /// O contêiner como fica depois do movimento.
    fn container(&self) -> &dyn Container;

    /// Produto movimentado, em base62.
    fn product_id(&self) -> &str;

    /// A linha do manifesto como fica, ou `None` se ela deixou de existir.
    fn cargo(&self) -> Option<&dyn ManifestCargo>;

    /// Se o manifesto inteiro deve ser apagado.
    ///
    /// Verdadeiro quando o desembarque esvaziou o contêiner: em vez de remover
    /// linha a linha, o manifesto vai junto.
    fn clear_manifest(&self) -> bool;

    /// O que registrar na telemetria.
    fn event(&self) -> TelemetryEvent;

    /// Desmonta a mudança e entrega o contêiner resultante.
    ///
    /// Existe porque quem responde ao movimento precisa **publicar** o contêiner
    /// no estado novo, e não só gravá-lo: a resposta de embarque leva o peso e o
    /// status atualizados. Consumir a mudança é o que evita reler do banco o que
    /// já está em memória — e, por consumi-la, garante que ninguém a persista de
    /// novo depois de tê-la publicado.
    fn into_container(self: Box<Self>) -> Box<dyn Container>;
}

/// A implementação do domínio de [`ManifestCargo`].
pub(crate) struct ManifestCargoModel {
    container_id: String,
    product_id: String,
    quantity: f64,
    weight: f64,
    created_at: DateTime<Utc>,
}

impl ManifestCargoModel {
    /// Monta uma linha de manifesto.
    pub(crate) fn new(
        container_id: String,
        product_id: String,
        quantity: f64,
        weight: f64,
    ) -> Self {
        Self {
            container_id,
            product_id,
            quantity,
            weight,
            created_at: Utc::now(),
        }
    }
}

impl ManifestCargo for ManifestCargoModel {
    fn container_id(&self) -> &str {
        &self.container_id
    }

    fn product_id(&self) -> &str {
        &self.product_id
    }

    fn quantity(&self) -> f64 {
        self.quantity
    }

    fn weight(&self) -> f64 {
        self.weight
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

/// A implementação do domínio de [`ManifestChange`].
pub(crate) struct ManifestChangeModel {
    container: Box<dyn Container>,
    product_id: String,
    cargo: Option<Box<dyn ManifestCargo>>,
    clear_manifest: bool,
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
