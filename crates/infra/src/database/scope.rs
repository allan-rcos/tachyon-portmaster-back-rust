//! O escopo de transação de uma requisição.
//!
//! A transação vive num task-local: ela é da **requisição**, não de um objeto
//! que alguém passa adiante. Abrir o escopo é o que faz `UnitOfWork::begin` ter
//! onde guardar a transação.
//!
//! > **Lacuna conhecida do `lint-exports`:** o `CURRENT` abaixo nasce de
//! > `tokio::task_local!`, e itens gerados por macro são invisíveis ao `syn`.
//! > O arquivo exporta dois itens na prática, não um — o `TransactionScope` e o
//! > task-local que só ele usa.

use std::future::Future;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::database::interno::transaction_slot::Slot;

tokio::task_local! {
    /// A transação da requisição corrente.
    ///
    /// Interno à `infra`: nem o `app` nem a apresentação alcançam este
    /// armazenamento. Se alcançassem, uma camada passaria a depender do formato
    /// interno da outra, e uma feature escrita só no `api` poderia esquecer de
    /// alimentá-lo — com o erro aparecendo em produção.
    pub(crate) static CURRENT: Slot;
}

/// O escopo em que uma transação pode existir.
///
/// Namespace, no molde do `Base62`: as duas operações são a mesma fronteira
/// vista de dois lados — abri-la e perguntar se ela está aberta.
pub struct TransactionScope;

impl TransactionScope {
    /// Executa `future` dentro de um escopo de transação.
    ///
    /// Fora deste escopo, `begin` falha em vez de abrir uma transação que
    /// ninguém conseguiria consultar depois. O `app` envolve nela a orquestração
    /// de cada caso de uso que toca o banco.
    pub async fn run<F: Future>(future: F) -> F::Output {
        CURRENT.scope(Arc::new(Mutex::new(None)), future).await
    }

    /// Se há um escopo de transação ativo.
    ///
    /// Não diz se há transação **aberta** — só se existe onde guardá-la. Serve
    /// para o `app` afirmar em teste que a sua moldura de transação de fato
    /// envolveu o caso de uso, em vez de descobrir que não envolveu quando o
    /// primeiro repositório falhar em execução.
    #[must_use]
    pub fn is_active() -> bool {
        CURRENT.try_with(|_| ()).is_ok()
    }
}
