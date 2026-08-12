//! O handle que abre a transação da tarefa.

use anyhow::{anyhow, Context as _};
use sqlx::MySqlPool;
use tokio::sync::OwnedMutexGuard;

use crate::config::InfraSecrets;
use crate::scope::database::intern::mariadb_context::MariaDbContext;
use crate::scope::database::intern::mariadb_pool::connect;
use crate::scope::database::mysql_transaction::{MySqlTransaction, TransactionGuard};
use crate::scope::intern::scope_slots::ScopeSlots;

/// O acesso ao banco que cada repositório carrega.
///
/// Sem estado por tarefa: o que ele guarda é o pool, que vale para o processo
/// inteiro. O estado da tarefa mora no [`MariaDbContext`], e este handle o
/// alcança pelo mapa do escopo — nunca ao contrário.
#[derive(Clone)]
pub(crate) struct MariaDbUnitOfWork {
    /// De onde a transação sai quando a primeira escrita a pede.
    pool: MySqlPool,
}

impl MariaDbUnitOfWork {
    /// Abre o pool e devolve o handle.
    ///
    /// Falhar aqui derruba o boot de propósito — melhor não subir do que subir
    /// com um banco inalcançável e descobrir isso na primeira requisição.
    pub(crate) async fn connect(secrets: &InfraSecrets) -> anyhow::Result<Self> {
        Ok(Self {
            pool: connect(secrets).await?,
        })
    }
}

impl MySqlTransaction for MariaDbUnitOfWork {
    async fn transaction(&self) -> anyhow::Result<TransactionGuard> {
        let slot = ScopeSlots::current::<MariaDbContext>()?.slot();
        let mut guard = slot.lock_owned().await;

        if guard.is_none() {
            *guard = Some(
                self.pool
                    .begin()
                    .await
                    .context("falha ao abrir transação no MariaDB")?,
            );
        }

        OwnedMutexGuard::try_map(guard, Option::as_mut)
            .map_err(|_| anyhow!("a transação sumiu do escopo entre abrir e emprestar"))
    }
}

#[cfg(test)]
#[path = "tests/mariadb_unit_of_work_test.rs"]
mod tests;
