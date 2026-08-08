//! A transação de um caso de uso.
//!
//! Todo caso de uso que toca o banco tem a mesma moldura: abrir escopo, `begin`,
//! fazer o trabalho, `commit` — e desfazer se qualquer passo falhar. No PHP essa
//! moldura era escrita à mão em cada caso de uso, o que significava lembrar de
//! chamar `rollback()` em **todos** os retornos antecipados. Um esquecimento não
//! quebrava teste nenhum: só deixava uma transação aberta segurando conexão do
//! pool até o timeout.
//!
//! Aqui a moldura é uma função. Não há como esquecer o `rollback`, porque não há
//! onde escrevê-lo.
//!
//! ## Por que o escopo é aberto aqui
//!
//! A transação vive num `task_local` da `infra`, e um `task_local` não pode ser
//! preenchido de dentro — alguém precisa envolver o future. É o `app` quem sabe
//! onde uma operação de negócio começa e termina, então é ele quem abre. Ver
//! [`TransactionScope::run`].

use crate::error::AppError;
use portmaster_infra::database::{TransactionScope, UnitOfWork};
use std::future::Future;

/// A moldura de transação de um caso de uso de escrita.
///
/// Namespace de uma operação só, no molde do `Base62`: `Transaction::run(...)`
/// diz o que está acontecendo no ponto de uso melhor que uma função livre
/// chamada `transaction(...)`.
pub(crate) struct Transaction;

impl Transaction {
    /// Executa `body` dentro de uma transação.
    ///
    /// Confirma se o corpo devolver `Ok`, desfaz se devolver `Err`. A falha do
    /// `rollback` é engolida de propósito: ela aconteceria **em cima** de um erro
    /// que já é a resposta, e trocar "a carga não cabe no contêiner" por "falha ao
    /// desfazer transação" esconderia do cliente a única das duas que ele pode
    /// resolver. O motivo real continua no `Err` que sai daqui.
    pub(crate) async fn run<U, T, F>(unit_of_work: &U, body: F) -> Result<T, AppError>
    where
        U: UnitOfWork,
        F: Future<Output = Result<T, AppError>>,
    {
        TransactionScope::run(async move {
            unit_of_work.begin().await?;

            match body.await {
                Ok(value) => {
                    unit_of_work.commit().await?;
                    Ok(value)
                }
                Err(error) => {
                    let _ = unit_of_work.rollback().await;
                    Err(error)
                }
            }
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use pretty_assertions::assert_eq;

    /// Uma unidade de trabalho que só conta o que foi pedido.
    #[derive(Clone, Default)]
    struct SpyUnitOfWork {
        begun: Arc<AtomicUsize>,
        committed: Arc<AtomicUsize>,
        rolled_back: Arc<AtomicUsize>,
    }

    impl UnitOfWork for SpyUnitOfWork {
        async fn begin(&self) -> anyhow::Result<()> {
            self.begun.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn commit(&self) -> anyhow::Result<()> {
            self.committed.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn rollback(&self) -> anyhow::Result<()> {
            self.rolled_back.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn o_sucesso_confirma_e_nao_desfaz() {
        let uow = SpyUnitOfWork::default();

        let value = Transaction::run(&uow, async { Ok::<_, AppError>(42) })
            .await
            .expect("o corpo não falhou");

        assert_eq!(value, 42);
        assert_eq!(uow.begun.load(Ordering::SeqCst), 1);
        assert_eq!(uow.committed.load(Ordering::SeqCst), 1);
        assert_eq!(uow.rolled_back.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn a_falha_desfaz_e_nao_confirma() {
        let uow = SpyUnitOfWork::default();

        let error = Transaction::run(&uow, async {
            Err::<(), _>(AppError::Conflict("não deu".into()))
        })
        .await
        .expect_err("o corpo falhou");

        assert!(matches!(error, AppError::Conflict(_)));
        assert_eq!(uow.rolled_back.load(Ordering::SeqCst), 1);
        assert_eq!(
            uow.committed.load(Ordering::SeqCst),
            0,
            "confirmar depois de falhar gravaria trabalho pela metade"
        );
    }

    #[tokio::test]
    async fn o_erro_do_corpo_sobrevive_ao_rollback() {
        // O caso que mais importa: o cliente precisa receber o motivo que ele
        // pode resolver, não o que aconteceu ao desfazer.
        let uow = SpyUnitOfWork::default();

        let error = Transaction::run(&uow, async {
            Err::<(), _>(AppError::not_found("produto", "abc"))
        })
        .await
        .expect_err("o corpo falhou");

        assert!(matches!(error, AppError::NotFound { id, .. } if id == "abc"));
    }

    #[tokio::test]
    async fn o_corpo_roda_dentro_do_escopo_de_transacao() {
        // Sem o escopo aberto, os repositórios não encontrariam a transação
        // corrente — e falhariam em execução, não em compilação.
        let uow = SpyUnitOfWork::default();

        let dentro = Transaction::run(&uow, async {
            Ok::<_, AppError>(TransactionScope::is_active())
        })
        .await
        .expect("o corpo não falhou");

        assert!(dentro);
    }
}
