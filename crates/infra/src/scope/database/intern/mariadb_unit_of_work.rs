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
mod tests {
    use super::*;
    use crate::scope::MasterScope;

    /// Fora do escopo não há onde guardar a transação, e pedir uma falha em vez
    /// de devolver algo inútil.
    ///
    /// É o que mantém o boot funcionando sem cerimônia: o que ele escreve é
    /// catálogo em memória, e nada ali pede transação.
    #[tokio::test]
    async fn fora_do_escopo_nao_ha_contexto_de_transacao() {
        let error = ScopeSlots::current::<MariaDbContext>()
            .err()
            .map(|error| error.to_string())
            .expect("deveria falhar");

        assert!(error.contains("nenhum escopo de tarefa ativo"));
    }

    /// Abrir o escopo não abre transação: ela só nasce quando alguém escreve.
    #[tokio::test]
    async fn o_escopo_nasce_sem_transacao() {
        MasterScope::run(|_| async {
            let context = ScopeSlots::current::<MariaDbContext>().expect("o banco está instalado");

            assert!(
                context.slot().lock().await.is_none(),
                "nenhuma consulta pediu a transação ainda"
            );
        })
        .await;
    }
}
