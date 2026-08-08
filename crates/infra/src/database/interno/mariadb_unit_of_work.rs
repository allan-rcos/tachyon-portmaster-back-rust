//! A transação por requisição sobre o `MariaDB`.

use anyhow::{anyhow, Context};
use sqlx::MySqlPool;

use crate::database::interno::transaction_guard::TransactionGuard;
use crate::database::interno::transaction_slot::Slot;
use crate::database::scope::CURRENT;
use crate::database::UnitOfWork;

/// A implementação sobre `MariaDB`.
pub(crate) struct MariadbUnitOfWork {
    /// De onde a transação sai quando um escopo abre.
    pool: MySqlPool,
}

impl MariadbUnitOfWork {
    /// Monta a unidade de trabalho sobre o pool do processo.
    pub(crate) const fn new(pool: MySqlPool) -> Self {
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

        Ok(TransactionGuard::new(guard))
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

    /// Desfaz a transação corrente, se houver.
    ///
    /// Sem transação é no-op **de propósito**: o caminho de erro chama rollback
    /// sem saber se chegou a abrir alguma, e transformar isso em falha
    /// esconderia o erro original atrás de um segundo.
    async fn rollback(&self) -> anyhow::Result<()> {
        let slot = Self::slot()?;
        let mut current = slot.lock().await;

        let Some(transaction) = current.take() else {
            return Ok(());
        };

        transaction
            .rollback()
            .await
            .context("falha ao desfazer transação no MariaDB")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::scope::TransactionScope;
    use std::sync::Arc;

    /// Sem o escopo aberto, pedir a transação corrente falha em vez de
    /// devolver algo inútil ou entrar em pânico.
    #[tokio::test]
    async fn fora_do_escopo_nao_ha_transacao() {
        let result = MariadbUnitOfWork::current().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn dentro_do_escopo_sem_begin_ainda_nao_ha_transacao() {
        TransactionScope::run(async {
            let result = MariadbUnitOfWork::current().await;
            assert!(result.is_err(), "abrir o escopo não abre a transação");
        })
        .await;
    }

    /// A garantia que sustenta o modelo: duas requisições simultâneas têm
    /// transações independentes, sem lock global entre elas.
    ///
    /// O slot sai de cada tarefa por clone, e não como endereço: se as duas
    /// tarefas apenas devolvessem um ponteiro, a primeira alocação já estaria
    /// liberada quando a segunda acontecesse, e o alocador poderia devolver o
    /// mesmo endereço — o teste acusaria uma mistura que não houve. Segurando
    /// os dois `Arc` vivos ao mesmo tempo, identidade distinta é garantida.
    #[tokio::test]
    async fn escopos_de_tarefas_diferentes_nao_se_misturam() {
        let first = tokio::spawn(TransactionScope::run(async { CURRENT.with(Clone::clone) }));
        let second = tokio::spawn(TransactionScope::run(async { CURRENT.with(Clone::clone) }));

        let (first, second) = tokio::join!(first, second);
        let first = first.expect("tarefa não deve entrar em pânico");
        let second = second.expect("tarefa não deve entrar em pânico");

        assert!(
            !Arc::ptr_eq(&first, &second),
            "cada escopo tem o próprio armazenamento"
        );
    }
}
