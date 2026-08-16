//! O handle que abre a transação da tarefa.

use anyhow::{anyhow, Context as _};
use sqlx::MySqlPool;
use tokio::sync::OwnedMutexGuard;

use crate::config::InfraSecrets;
use crate::scope::database::intern::mariadb_context::MariaDbContext;
use crate::scope::database::intern::mariadb_pool::open;
use crate::scope::database::mysql_transaction::{MySqlTransaction, TransactionGuard};
use crate::scope::intern::scope_slots::ScopeSlots;

/// Monta o handle sobre um pool preguiçoso.
///
/// Síncrono: abrir o pool não toca a rede, e o que pode falhar aqui é o parse
/// da URI e a checagem de TLS, que já eram síncronos. Quem confirma que o banco
/// responde é o [`MariaDbUnitOfWork::ping`], no boot.
///
/// Devolve o tipo concreto porque o provider precisa guardá-lo num `static`, e
/// um `static` exige um tipo com nome — um `impl Trait` não é um. Nem a função
/// nem o tipo saem do módulo `scope::database`.
pub(in crate::scope::database) fn mariadb_unit_of_work(
    secrets: &InfraSecrets,
) -> anyhow::Result<MariaDbUnitOfWork> {
    Ok(MariaDbUnitOfWork {
        pool: open(secrets)?,
    })
}

/// O acesso ao banco que cada repositório carrega.
///
/// Sem estado por tarefa: o que ele guarda é o pool, que vale para o processo
/// inteiro. O estado da tarefa mora no [`MariaDbContext`], e este handle o
/// alcança pelo mapa do escopo — nunca ao contrário.
#[derive(Clone)]
pub(in crate::scope::database) struct MariaDbUnitOfWork {
    /// De onde a transação sai quando a primeira escrita a pede.
    pool: MySqlPool,
}

impl MariaDbUnitOfWork {
    /// Confirma que o banco responde, e derruba o boot se não responder.
    ///
    /// É a primeira conexão de verdade do processo: o pool nasce preguiçoso, e
    /// até aqui nada tocou a rede. Falhar é o desfecho desejado — subir com um
    /// banco inalcançável só adia a descoberta para a primeira requisição, com
    /// o processo já reportado como saudável.
    pub(in crate::scope::database) async fn ping(&self) -> anyhow::Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .context("o MariaDB não respondeu ao SELECT de sanidade")?;

        Ok(())
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
