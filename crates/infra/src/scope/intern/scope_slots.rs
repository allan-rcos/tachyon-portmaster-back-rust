//! O mapa de contextos da tarefa corrente.
//!
//! Há **um** task-local no sistema, e é este. É o que mata o aninhamento: por
//! mais contextos que existam, o escopo faz uma única chamada de
//! `LocalKey::scope`, e não um nível de indentação por participante.
//!
//! > **Lacuna conhecida do `lint-exports`:** o `CURRENT` abaixo nasce de
//! > `tokio::task_local!`, e itens gerados por macro são invisíveis ao `syn`.
//! > O arquivo exporta dois itens na prática, não um — o `ScopeSlots` e o
//! > task-local que só ele usa.

use std::any::{Any, TypeId};
use std::future::Future;
use std::sync::Arc;

use anyhow::anyhow;

use crate::scope::scope_context::ScopeContext;

tokio::task_local! {
    /// Os contextos da tarefa corrente.
    ///
    /// Interno à `infra`: nem o `app` nem a apresentação alcançam este
    /// armazenamento. Se alcançassem, uma camada passaria a depender do formato
    /// interno da outra, e uma feature escrita só no `api` poderia esquecer de
    /// alimentá-lo — com o erro aparecendo em produção.
    pub(crate) static CURRENT: Arc<ScopeSlots>;
}

/// Os contextos que um escopo instalou, indexados de duas formas.
///
/// O mesmo `Arc` entra nas duas listas: por tipo, para o dono recuperar o
/// contexto dele com o tipo de volta; e em ordem de instalação, para a unidade
/// de trabalho percorrer todos sem saber de nenhum.
#[derive(Default)]
pub(crate) struct ScopeSlots {
    /// Por tipo, para quem quer o **seu** contexto.
    by_type: Vec<(TypeId, Arc<dyn Any + Send + Sync>)>,
    /// Em ordem de instalação, para quem confirma **todos**.
    all: Vec<Arc<dyn ScopeContext>>,
}

impl ScopeSlots {
    /// Registra o contexto de uma camada.
    pub(crate) fn put<T: ScopeContext>(&mut self, context: T) {
        let context = Arc::new(context);

        self.by_type.push((TypeId::of::<T>(), context.clone()));
        self.all.push(context);
    }

    /// Os contextos instalados, para quem confirma.
    pub(crate) fn contexts(&self) -> Vec<Arc<dyn ScopeContext>> {
        self.all.clone()
    }

    /// Roda `future` com estes contextos instalados na tarefa.
    pub(crate) async fn install<F: Future>(self, future: F) -> F::Output {
        CURRENT.scope(Arc::new(self), future).await
    }

    /// O contexto de `T` na tarefa corrente.
    ///
    /// Falha fora de um escopo em vez de devolver algo inútil, e falha de novo
    /// se o escopo existir sem esta camada — o que só acontece se alguém
    /// declarar o contexto e esquecer o `#[distributed_slice]`.
    pub(crate) fn current<T: ScopeContext>() -> anyhow::Result<Arc<T>> {
        let slots = CURRENT
            .try_with(Arc::clone)
            .map_err(|_| anyhow!("nenhum escopo de tarefa ativo"))?;

        slots
            .by_type
            .iter()
            .find(|(id, _)| *id == TypeId::of::<T>())
            .and_then(|(_, context)| Arc::clone(context).downcast::<T>().ok())
            .ok_or_else(|| anyhow!("a camada não foi instalada neste escopo"))
    }

    /// Se há escopo ativo.
    ///
    /// Não diz se há transação **aberta** — só se existe onde guardá-la.
    pub(crate) fn is_active() -> bool {
        CURRENT.try_with(|_| ()).is_ok()
    }
}
