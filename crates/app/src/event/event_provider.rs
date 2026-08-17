//! Quem serve a pilha de eventos.

use crate::event::intern::meta_event_stack::MetaEventStack;
use crate::event::meta_event_stack_publisher::MetaEventStackPublisher;
use crate::event::meta_event_stack_subscriber::MetaEventStackSubscriber;

/// A pilha de eventos, já como contrato.
///
/// Namespace estático, no molde do ADR 0011. Não há o que memoizar: o estado
/// mora na tarefa, e o que este factory devolve é um ZST — construí-lo a cada
/// chamada custa exatamente nada.
///
/// Devolve os **dois** contratos de uma vez porque é um objeto só visto de dois
/// lados. Quem o recebe escolhe qual metade declara: o `ServicesProvider` o
/// entrega a um caso de uso como publisher, e o `AppProvider` o entrega à
/// apresentação como subscriber — e o `pub(crate)` do publisher garante que a
/// segunda metade não atravesse a fronteira do crate.
pub(crate) struct EventProvider;

impl EventProvider {
    /// A pilha de eventos da tarefa corrente.
    pub(crate) fn meta_event_stack(
    ) -> impl MetaEventStackSubscriber + MetaEventStackPublisher + Sync + Clone + use<> + 'static
    {
        MetaEventStack
    }
}
