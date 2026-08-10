//! O contrato de leitura do efeito de um movimento de carga.

use crate::domain::{Container, ManifestCargo};
use crate::enums::TelemetryEvent;

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
