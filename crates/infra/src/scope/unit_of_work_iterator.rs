//! A unidade de trabalho que percorre os contextos do escopo.

use std::sync::Arc;

use crate::scope::scope_context::{Closing, ScopeContext};
use crate::scope::UnitOfWork;

/// Confirma e desfaz **todos** os contextos da tarefa, seja qual for a camada.
///
/// É o que o [`MasterScope::run`](crate::scope::MasterScope::run) entrega ao
/// corpo do caso de uso. O `app` recebe este tipo concreto e chama métodos de
/// instância — o despacho dinâmico existe uma camada abaixo, na lista, porque
/// uma slice preenchida pelo linker é homogênea e não há como coletar N tipos de
/// contexto sem apagar alguma coisa.
#[derive(Clone)]
pub struct UnitOfWorkIterator {
    /// Os contextos instalados neste escopo, na ordem em que entraram.
    contexts: Vec<Arc<dyn ScopeContext>>,
}

impl UnitOfWorkIterator {
    /// Monta o iterador sobre os contextos de um escopo.
    pub(crate) const fn new(contexts: Vec<Arc<dyn ScopeContext>>) -> Self {
        Self { contexts }
    }
}

impl UnitOfWork for UnitOfWorkIterator {
    async fn commit(&self) -> anyhow::Result<()> {
        close_all(&self.contexts, ScopeContext::commit).await
    }

    async fn rollback(&self) -> anyhow::Result<()> {
        close_all(&self.contexts, ScopeContext::rollback).await
    }
}

/// Fecha todos os contextos e reporta a primeira falha.
///
/// Todos, e não até a primeira falha: um contexto que ficasse aberto porque o
/// anterior recusou seguraria a conexão até o fim do processo. A primeira falha
/// é a reportada porque é a que explica as seguintes.
async fn close_all<'a, F>(contexts: &'a [Arc<dyn ScopeContext>], close: F) -> anyhow::Result<()>
where
    F: Fn(&'a dyn ScopeContext) -> Closing<'a>,
{
    let mut failure = None;

    for context in contexts {
        if let Err(error) = close(context.as_ref()).await {
            failure.get_or_insert(error);
        }
    }

    failure.map_or(Ok(()), Err)
}
