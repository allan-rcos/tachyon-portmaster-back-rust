//! O que a tarefa escreveu em memória e ainda não publicou.

use std::collections::BTreeSet;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::scope::intern::scope_slots::ScopeSlots;
use crate::scope::memory::intern::pending_write::{Operation, PendingWrite};
use crate::scope::scope_context::{Closing, ScopeContext};

/// As escritas em memória da tarefa corrente.
///
/// Existe para que uma escrita em memória feita dentro de um caso de uso siga a
/// mesma sorte da escrita em banco: se o SQL não confirmar, o cache não publica.
/// Sem isto, um erro no meio da transação deixaria memória e banco discordando,
/// e o processo continuaria de pé servindo a discordância.
pub(crate) struct MokaContext {
    /// A fila, na ordem em que a tarefa escreveu.
    pending: Mutex<Vec<PendingWrite>>,
}

impl MokaContext {
    /// Registra o contexto da memória na tarefa.
    pub(super) fn install(slots: &mut ScopeSlots) {
        slots.put(Self {
            pending: Mutex::default(),
        });
    }

    /// Enfileira uma escrita da tarefa.
    pub(super) async fn enqueue(&self, write: PendingWrite) {
        self.pending.lock().await.push(write);
    }

    /// O que a própria tarefa já escreveu nesta entrada.
    ///
    /// `None` significa "a fila não tem nada a dizer, pergunte ao cache";
    /// `Some(None)`, que a tarefa apagou a entrada. Sem esta consulta, gravar e
    /// ler na mesma tarefa devolveria o valor anterior — e nenhum repositório do
    /// `MariaDB` se comporta assim.
    pub(super) async fn lookup(
        &self,
        store: &str,
        group: &str,
        key: &str,
    ) -> Option<Option<Arc<Vec<u8>>>> {
        let pending = self.pending.lock().await;

        pending
            .iter()
            .rev()
            .filter(|write| write.store == store && write.group == group)
            .find_map(|write| match &write.operation {
                Operation::Put {
                    key: written,
                    value,
                    ..
                } if written == key => Some(Some(Arc::clone(value))),
                Operation::Put { .. } => None,
                Operation::ClearGroup => Some(None),
            })
    }

    /// Aplica sobre `keys` o que a tarefa acrescentou ou tirou do grupo.
    pub(super) async fn overlay(&self, store: &str, group: &str, keys: &mut BTreeSet<String>) {
        let pending = self.pending.lock().await;

        for write in pending
            .iter()
            .filter(|write| write.store == store && write.group == group)
        {
            match &write.operation {
                Operation::Put { key, .. } => {
                    keys.insert(key.clone());
                }
                Operation::ClearGroup => keys.clear(),
            }
        }
    }
}

impl ScopeContext for MokaContext {
    fn commit(&self) -> Closing<'_> {
        Box::pin(async {
            let pending = std::mem::take(&mut *self.pending.lock().await);

            for write in pending {
                write.apply().await?;
            }

            Ok(())
        })
    }

    /// Descarta o que a tarefa escreveu.
    ///
    /// Barato porque nada foi publicado: a fila é a única cópia, e soltá-la é o
    /// desfazer inteiro.
    fn rollback(&self) -> Closing<'_> {
        Box::pin(async {
            self.pending.lock().await.clear();
            Ok(())
        })
    }
}
