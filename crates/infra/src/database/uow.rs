//! A transação da requisição.
//!
//! Quem **abre e fecha** é o `app`, porque é ele que sabe onde uma operação de
//! negócio começa e termina. Quem **usa** são os repositórios, que obtêm a
//! transação corrente por um acessor interno. O `app` nunca vê a transação em
//! si: ele só chama `begin`/`commit`, e por isso não tem como passar a conexão
//! adiante nem executar SQL por fora do repositório.
//!
//! ## Onde a transação mora
//!
//! Num `tokio::task_local!`, que é isolado por requisição sem custo de lock
//! global. Duas escolhas aqui merecem explicação, porque a alternativa óbvia não
//! funciona:
//!
//! **Por que `tokio::sync::Mutex` e não `RefCell`.** Executar SQL exige o
//! empréstimo mutável da transação **atravessando um `.await`**. Um `RefMut` de
//! `RefCell` não é `Send`, o que tornaria todo handler `!Send` — e o axum exige
//! futures `Send`. O mutex assíncrono devolve um guard que atravessa `.await`
//! sem perder `Send`.
//!
//! **Por que existe [`scope`].** Um `task_local` não pode ser preenchido de
//! dentro: só se entra nele envolvendo um future. Então `begin()` sozinho não
//! consegue criar o próprio armazenamento — alguém precisa abrir o escopo antes.
//! É o que [`scope`] faz, e é o `app` quem a chama, ao redor da orquestração de
//! cada caso de uso.

use std::future::Future;
use std::sync::Arc;

use anyhow::{anyhow, Context};
use sqlx::mysql::MySql;
use sqlx::{MySqlPool, Transaction};
use tokio::sync::Mutex;

/// Gerencia o ciclo da transação — e nada mais.
///
/// Não executa consulta: o que ela oferece é começo, fim e desistência. Manter a
/// execução fora daqui é o que impede o `app` de rodar SQL.
#[trait_variant::make(Send)]
pub trait UnitOfWork {
    /// Abre a transação.
    async fn begin(&self) -> anyhow::Result<()>;

    /// Confirma o que foi feito.
    async fn commit(&self) -> anyhow::Result<()>;

    /// Descarta o que foi feito.
    ///
    /// Idempotente: chamar sem transação aberta não é erro, porque o caminho de
    /// falha frequentemente não sabe se chegou a abrir uma.
    async fn rollback(&self) -> anyhow::Result<()>;
}

/// O que o `task_local` guarda.
type Slot = Arc<Mutex<Option<Transaction<'static, MySql>>>>;

tokio::task_local! {
    /// A transação da requisição corrente.
    ///
    /// Interno à `infra`: nem o `app` nem a apresentação alcançam este
    /// armazenamento. Se alcançassem, uma camada passaria a depender do formato
    /// interno da outra, e uma feature escrita só no `api` poderia esquecer de
    /// alimentá-lo — com o erro aparecendo em produção.
    static CURRENT: Slot;
}

/// Executa `future` dentro de um escopo de transação.
///
/// Fora deste escopo, `begin` falha em vez de abrir uma transação que ninguém
/// conseguiria consultar depois. O `app` envolve nela a orquestração de cada
/// caso de uso que toca o banco.
pub async fn scope<F: Future>(future: F) -> F::Output {
    CURRENT.scope(Arc::new(Mutex::new(None)), future).await
}

/// Se há um escopo de transação ativo.
///
/// Não diz se há transação **aberta** — só se existe onde guardá-la. Serve para
/// o `app` afirmar em teste que a sua moldura de transação de fato envolveu o
/// caso de uso, em vez de descobrir que não envolveu quando o primeiro
/// repositório falhar em execução.
pub fn in_scope() -> bool {
    CURRENT.try_with(|_| ()).is_ok()
}

/// A implementação sobre MariaDB.
pub(crate) struct MariadbUnitOfWork {
    pool: MySqlPool,
}

impl MariadbUnitOfWork {
    /// Monta a unidade de trabalho sobre o pool do processo.
    pub(crate) fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// A transação corrente, para os repositórios.
    ///
    /// `pub(crate)`: é o que mantém a conexão invisível para fora da `infra`.
    pub(crate) async fn current() -> anyhow::Result<TransactionGuard> {
        let slot = CURRENT
            .try_with(Clone::clone)
            .map_err(|_| anyhow!("nenhum escopo de transação ativo"))?;

        let guard = slot.lock_owned().await;
        if guard.is_none() {
            return Err(anyhow!("nenhuma transação aberta: falta um begin"));
        }

        Ok(TransactionGuard { guard })
    }

    /// O slot da requisição corrente.
    fn slot() -> anyhow::Result<Slot> {
        CURRENT
            .try_with(Clone::clone)
            .map_err(|_| anyhow!("nenhum escopo de transação ativo"))
    }
}

impl UnitOfWork for MariadbUnitOfWork {
    async fn begin(&self) -> anyhow::Result<()> {
        let slot = Self::slot()?;
        let mut current = slot.lock().await;

        if current.is_some() {
            return Err(anyhow!("já há uma transação aberta nesta requisição"));
        }

        *current = Some(
            self.pool
                .begin()
                .await
                .context("falha ao abrir transação no MariaDB")?,
        );

        Ok(())
    }

    async fn commit(&self) -> anyhow::Result<()> {
        let slot = Self::slot()?;
        let mut current = slot.lock().await;

        let transaction = current
            .take()
            .ok_or_else(|| anyhow!("commit sem transação aberta"))?;

        transaction
            .commit()
            .await
            .context("falha ao confirmar transação no MariaDB")
    }

    async fn rollback(&self) -> anyhow::Result<()> {
        let slot = Self::slot()?;
        let mut current = slot.lock().await;

        // Sem transação é no-op de propósito: o caminho de erro chama rollback
        // sem saber se chegou a abrir alguma, e transformar isso em falha
        // esconderia o erro original atrás de um segundo.
        let Some(transaction) = current.take() else {
            return Ok(());
        };

        transaction
            .rollback()
            .await
            .context("falha ao desfazer transação no MariaDB")
    }
}

/// Acesso emprestado à transação corrente.
///
/// Enquanto vive, nenhuma outra parte da mesma requisição consegue a transação —
/// que é o correto, já que uma transação SQL é sequencial por natureza.
pub(crate) struct TransactionGuard {
    guard: tokio::sync::OwnedMutexGuard<Option<Transaction<'static, MySql>>>,
}

impl TransactionGuard {
    /// A transação, para passar ao `sqlx`.
    pub(crate) fn as_mut(&mut self) -> &mut Transaction<'static, MySql> {
        // `current()` já garantiu que há transação, e o guard impede que alguém
        // a tire enquanto este empréstimo vive.
        self.guard
            .as_mut()
            .expect("o guard só é construído com transação presente")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fora_do_escopo_nao_ha_transacao() {
        // Sem o escopo aberto, pedir a transação corrente falha em vez de
        // devolver algo inútil ou entrar em pânico.
        let result = MariadbUnitOfWork::current().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn dentro_do_escopo_sem_begin_ainda_nao_ha_transacao() {
        scope(async {
            let result = MariadbUnitOfWork::current().await;
            assert!(result.is_err(), "abrir o escopo não abre a transação");
        })
        .await;
    }

    #[tokio::test]
    async fn escopos_de_tarefas_diferentes_nao_se_misturam() {
        // A garantia que sustenta o modelo: duas requisições simultâneas têm
        // transações independentes, sem lock global entre elas.
        //
        // O slot sai de cada tarefa por clone, e não como endereço: se as duas
        // tarefas apenas devolvessem um ponteiro, a primeira alocação já estaria
        // liberada quando a segunda acontecesse, e o alocador poderia devolver o
        // mesmo endereço — o teste acusaria uma mistura que não houve. Segurando
        // os dois `Arc` vivos ao mesmo tempo, identidade distinta é garantida.
        let first = tokio::spawn(scope(async { CURRENT.with(Clone::clone) }));
        let second = tokio::spawn(scope(async { CURRENT.with(Clone::clone) }));

        let (first, second) = tokio::join!(first, second);
        let first = first.expect("tarefa não deve entrar em pânico");
        let second = second.expect("tarefa não deve entrar em pânico");

        assert!(
            !Arc::ptr_eq(&first, &second),
            "cada escopo tem o próprio armazenamento"
        );
    }
}
