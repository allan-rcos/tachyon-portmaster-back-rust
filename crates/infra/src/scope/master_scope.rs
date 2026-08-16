//! O escopo de tarefa que todo caso de uso abre.

use std::future::Future;

use crate::scope::intern::scope_slots::ScopeSlots;
use crate::scope::scope_layers::SCOPE_LAYERS;
use crate::scope::{UnitOfWork, UnitOfWorkIterator};

/// O escopo em que os contextos por tarefa existem.
///
/// Namespace, no molde do `Base62`: as duas operações são a mesma fronteira
/// vista de dois lados — abri-la e perguntar se ela está aberta.
pub struct MasterScope;

impl MasterScope {
    /// Abre o escopo da tarefa e roda `body` dentro dele.
    ///
    /// O corpo recebe a unidade de trabalho e é ele quem confirma. Não confirmar
    /// é o rollback: sair por `?` desfaz tudo, e esquecer o commit desfaz tudo
    /// também. É o que apaga o `rollback` explícito de cada braço de erro.
    ///
    /// O tamanho desta função não depende de quantos contextos existem, e é esse
    /// o ponto: acrescentar um não toca em nada aqui.
    pub async fn run<B, F>(body: B) -> F::Output
    where
        B: FnOnce(UnitOfWorkIterator) -> F,
        F: Future,
    {
        let mut slots = ScopeSlots::default();
        for layer in SCOPE_LAYERS {
            (layer.install)(&mut slots);
        }

        let unit_of_work = UnitOfWorkIterator::new(slots.contexts());
        let closing = unit_of_work.clone();

        slots
            .install(async move {
                let output = body(unit_of_work).await;
                let _ = closing.rollback().await;
                output
            })
            .await
    }

    /// Se há um escopo ativo.
    ///
    /// Serve para o `app` afirmar em teste que a sua moldura de fato envolveu o
    /// caso de uso, em vez de descobrir que não envolveu quando o primeiro
    /// repositório falhar em execução.
    #[must_use]
    pub fn is_active() -> bool {
        ScopeSlots::is_active()
    }
}
