//! A transação da tarefa corrente.

use std::sync::Arc;

use anyhow::Context as _;
use sqlx::mysql::MySql;
use sqlx::Transaction;
use tokio::sync::Mutex;

use crate::scope::intern::scope_slots::ScopeSlots;
use crate::scope::scope_context::{Closing, ScopeContext};

/// Onde a transação da tarefa vive.
///
/// `Arc` porque o empréstimo que os repositórios recebem é `Owned` e sobrevive
/// à referência ao contexto; `Mutex` assíncrono porque duas chamadas
/// concorrentes da mesma tarefa podem pedir a transação, e uma transação SQL é
/// sequencial por natureza; `Option` porque o escopo nasce sem transação e pode
/// morrer sem nunca ter aberto uma.
pub(super) type Slot = Arc<Mutex<Option<Transaction<'static, MySql>>>>;

/// A transação da tarefa corrente.
pub(crate) struct MariaDbContext {
    /// A transação, quando alguma chegou a ser aberta.
    slot: Slot,
}

impl MariaDbContext {
    /// Registra o contexto do banco na tarefa.
    pub(super) fn install(slots: &mut ScopeSlots) {
        slots.put(Self {
            slot: Slot::default(),
        });
    }

    /// O slot da tarefa, para quem abre a transação.
    pub(super) fn slot(&self) -> Slot {
        Arc::clone(&self.slot)
    }
}

impl ScopeContext for MariaDbContext {
    fn commit(&self) -> Closing<'_> {
        Box::pin(async {
            let transaction = self.slot.lock().await.take();

            let Some(transaction) = transaction else {
                return Ok(());
            };

            transaction
                .commit()
                .await
                .context("falha ao confirmar transação no MariaDB")
        })
    }

    /// Desfaz a transação da tarefa, se houver.
    ///
    /// O escopo chama isto na saída sem saber se o corpo confirmou. Sem
    /// transação é no-op **de propósito**: o caminho de erro não sabe se chegou
    /// a abrir alguma, e transformar isso em falha esconderia o erro original
    /// atrás de um segundo.
    ///
    /// Cancelamento (cliente desconecta, timeout do tower) não passa por aqui,
    /// porque não há `Drop` assíncrono. Não é vazamento: o `Drop` da
    /// `sqlx::Transaction` enfileira o rollback na conexão ao devolvê-la ao
    /// pool.
    fn rollback(&self) -> Closing<'_> {
        Box::pin(async {
            let transaction = self.slot.lock().await.take();

            let Some(transaction) = transaction else {
                return Ok(());
            };

            transaction
                .rollback()
                .await
                .context("falha ao desfazer transação no MariaDB")
        })
    }
}
