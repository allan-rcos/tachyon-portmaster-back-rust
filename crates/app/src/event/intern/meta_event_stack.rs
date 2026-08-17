//! A pilha de eventos da tarefa corrente.
//!
//! > **Lacuna conhecida do `lint-exports`:** o `STACK` abaixo nasce de
//! > `tokio::task_local!`, e itens gerados por macro são invisíveis ao `syn`.
//! > O arquivo exporta dois itens na prática, não um — o `MetaEventStack` e o
//! > task-local que só ele alcança.
//!
//! ## A pilha é uma máscara de bits
//!
//! Um `u8`, um bit por evento. Não há alocação, não há `RefCell` e não há
//! rastreio de empréstimo em execução: um `Cell` de um valor `Copy` se lê e se
//! escreve inteiro.
//!
//! O preço é o teto de oito eventos e a perda da multiplicidade — emitir o mesmo
//! evento duas vezes é indistinguível de emiti-lo uma. Nenhuma das duas coisas
//! importa para quem pergunta "isto aconteceu?", que é a única pergunta que os
//! `captured` do sistema fazem.
//!
//! A largura é **privada a este arquivo**. Nem o enum, nem os dois traits, nem
//! os middlewares, nem os casos de uso a enxergam — trocar `u8` por `u16` é uma
//! edição local, e a [`bit`] logo abaixo garante que ela não seja esquecida.

use std::cell::Cell;
use std::future::Future;

use crate::event::meta_event::MetaEvent;
use crate::event::meta_event_stack_publisher::MetaEventStackPublisher;
use crate::event::meta_event_stack_subscriber::MetaEventStackSubscriber;

tokio::task_local! {
    /// Os eventos registrados nesta tarefa, um bit por evento.
    ///
    /// Interno a este módulo: de fora se chega pelos dois traits, e nenhum deles
    /// revela que existe uma máscara aqui.
    static STACK: Cell<u8>;
}

/// O bit de cada evento.
///
/// O `match` é exaustivo de propósito, e é a guarda do teto: acrescentar uma
/// variante a [`MetaEvent`] derruba a compilação **aqui**, que é o único lugar
/// que sabe a largura da máscara. É o que transforma o nono evento num
/// alargamento consciente de `u8` para `u16`, em vez de um bit que sai do byte
/// em silêncio.
const fn bit(event: MetaEvent) -> u8 {
    match event {
        MetaEvent::ViewCacheHit => 1 << 0,
    }
}

/// A pilha de eventos, vista dos dois lados.
///
/// ZST: não guarda nada, porque o estado é da tarefa e não dele. Injetá-lo custa
/// o mesmo que não injetar, e o que se ganha é o caso de uso depender de um
/// contrato em vez de alcançar um task-local de outro módulo.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct MetaEventStack;

impl MetaEventStack {
    /// Aplica `change` à máscara, se houver escopo.
    ///
    /// O `Err` do `try_with` é descartado, e é só isto que faz toda operação
    /// desta pilha virar no-op fora de um escopo aberto.
    fn update(change: impl FnOnce(u8) -> u8) {
        let _ = STACK.try_with(|mask| mask.set(change(mask.get())));
    }
}

impl MetaEventStackSubscriber for MetaEventStack {
    fn scope<F>(&self, future: F) -> impl Future<Output = F::Output> + Send
    where
        F: Future + Send,
    {
        STACK.scope(Cell::new(0), future)
    }

    fn flush(&self, event: Option<MetaEvent>) {
        Self::update(|mask| event.map_or(0, |event| mask & !bit(event)));
    }

    fn captured(&self, event: MetaEvent) -> bool {
        STACK
            .try_with(|mask| mask.get() & bit(event) != 0)
            .unwrap_or(false)
    }
}

impl MetaEventStackPublisher for MetaEventStack {
    fn emit(&self, event: MetaEvent) {
        Self::update(|mask| mask | bit(event));
    }
}
