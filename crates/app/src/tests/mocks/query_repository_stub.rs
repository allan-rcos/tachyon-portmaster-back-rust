//! O stub de [`QueryRepository`].

use portmaster_infra::query::{Dql, QueryRepository};
use std::any::Any;

/// A execução de DQL, armada com a View que a consulta deve devolver.
///
/// ## Por que stub, e não `mock!` como as outras ports
///
/// `run` é genérico sobre o DQL, e o `mockall` arma a expectativa nomeando esse
/// tipo: `expect_run::<D>()`. Só que cada `dql::*` devolve `impl SqlDql<…>` —
/// um tipo anônimo, que nenhum teste consegue escrever. A expectativa não tem
/// como ser armada.
///
/// Então o stub não olha para o DQL: guarda uma View e a entrega a quem pedir,
/// conferindo pelo [`Any`] que o tipo pedido é o que foi armado. Uma consulta
/// que peça outra View falha em vez de devolver algo inventado.
pub(crate) struct StubQueries {
    /// A View armada, apagada porque o tipo só se conhece no ponto da chamada.
    view: Box<dyn Fn() -> Box<dyn Any + Send> + Send + Sync>,
}

impl StubQueries {
    /// Um stub que devolve esta View a cada consulta.
    pub(crate) fn returning<V: Clone + Send + Sync + 'static>(view: V) -> Self {
        Self {
            view: Box::new(move || Box::new(view.clone())),
        }
    }

    /// Um stub que nunca deve ser consultado.
    ///
    /// Pedir uma consulta a ele é o próprio defeito — é assim que um teste
    /// afirma "o cache respondeu, o banco não foi tocado".
    pub(crate) fn never() -> Self {
        Self {
            view: Box::new(|| panic!("o banco não deveria ter sido consultado")),
        }
    }
}

impl QueryRepository for StubQueries {
    async fn run<D: Dql + Send + 'static>(&self, _dql: D) -> anyhow::Result<D::View> {
        (self.view)()
            .downcast::<D::View>()
            .map(|view| *view)
            .map_err(|_| anyhow::anyhow!("o stub foi armado com outra View"))
    }
}
