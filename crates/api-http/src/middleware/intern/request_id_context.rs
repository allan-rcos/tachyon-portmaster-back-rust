//! O id de correlação da tarefa corrente.
//!
//! > **Lacuna conhecida do `lint-exports`:** o `CURRENT` abaixo nasce de
//! > `tokio::task_local!`, e itens gerados por macro são invisíveis ao `syn`.
//! > O arquivo exporta dois itens na prática, não um — o `RequestIdContext` e o
//! > task-local que só ele lê.

use std::future::Future;

use crate::middleware::request_id_port::RequestIdPort;

tokio::task_local! {
    /// O id da requisição corrente.
    ///
    /// Interno a este módulo: quem o alcança de fora passa pelo
    /// [`RequestIdPort`], que só lê. Escrever ficou com o layer irmão, que é o
    /// único que sabe de onde o id vem.
    static CURRENT: String;
}

/// O adaptador que serve o id do escopo corrente.
///
/// ZST: não guarda nada, porque o estado é da tarefa e não dele. Injetá-lo custa
/// o mesmo que não injetar, e o que se ganha é o controller depender de um
/// contrato em vez de alcançar um task-local de outro módulo.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RequestIdContext;

impl RequestIdContext {
    /// Roda `future` com este id instalado na tarefa.
    ///
    /// `pub(super)`: é o escritor, e só o layer irmão o alcança. Um task-local
    /// não pode ser escrito de dentro — só envolvido —, então esta é a única
    /// forma de o id existir, e ela está fechada para fora do módulo.
    pub(super) async fn scope<F: Future>(id: String, future: F) -> F::Output {
        CURRENT.scope(id, future).await
    }
}

impl RequestIdPort for RequestIdContext {
    fn current(&self) -> Option<String> {
        CURRENT.try_with(Clone::clone).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn dentro_do_escopo_o_id_esta_disponivel() {
        RequestIdContext::scope("abc123".to_owned(), async {
            assert_eq!(RequestIdContext.current(), Some("abc123".to_owned()));
        })
        .await;
    }

    /// Faltar id não é erro: correlação ausente não impede ninguém de responder.
    #[tokio::test]
    async fn fora_do_escopo_nao_ha_id() {
        assert_eq!(RequestIdContext.current(), None);
    }

    #[tokio::test]
    async fn escopos_de_requisicoes_diferentes_nao_se_misturam() {
        let first = tokio::spawn(RequestIdContext::scope("um".to_owned(), async {
            RequestIdContext.current()
        }));
        let second = tokio::spawn(RequestIdContext::scope("dois".to_owned(), async {
            RequestIdContext.current()
        }));

        let (first, second) = tokio::join!(first, second);

        assert_eq!(
            first.expect("a tarefa não entra em pânico"),
            Some("um".to_owned())
        );
        assert_eq!(
            second.expect("a tarefa não entra em pânico"),
            Some("dois".to_owned())
        );
    }
}
