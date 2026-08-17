//! O lado de escrita da pilha de eventos.

use crate::event::meta_event::MetaEvent;

/// Registra um evento na pilha da tarefa corrente.
///
/// `pub(crate)` de propósito, e é o que sustenta a regra: **só um caso de uso
/// emite**. O trait não sai do `app`, então nem a apresentação nem um crate de
/// fora conseguem sequer nomear o que precisariam implementar ou injetar para
/// escrever na pilha. Ler é público — ver
/// [`MetaEventStackSubscriber`](crate::event::MetaEventStackSubscriber).
///
/// Fora de um escopo aberto, emitir não faz nada.
pub(crate) trait MetaEventStackPublisher {
    /// Registra o evento, se houver escopo.
    fn emit(&self, event: MetaEvent);
}
