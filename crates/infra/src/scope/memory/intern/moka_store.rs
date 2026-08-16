//! O handle de memória que cada repositório carrega.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use crate::scope::intern::scope_slots::ScopeSlots;
use crate::scope::memory::intern::moka_cache::MokaCache;
use crate::scope::memory::intern::moka_context::MokaContext;
use crate::scope::memory::intern::pending_write::{Operation, PendingWrite};
use crate::scope::memory::memory_store::MemoryStore;

/// Monta o handle sobre um mapa do processo.
///
/// O `name` não é enfeite: ele é o que distingue duas escritas na fila da
/// tarefa. Dois stores podem ter grupo e chave iguais e significar coisas
/// diferentes — é exatamente o caso da permissão e do grupo de marcador, que o
/// schema anterior separava em duas tabelas.
pub(in crate::scope::memory) fn moka_store(
    name: &'static str,
    cache: MokaCache,
) -> impl MemoryStore + Sync + Clone + use<> + 'static {
    MokaStore { name, cache }
}

/// Um recorte da memória do processo.
#[derive(Clone)]
struct MokaStore {
    /// Que recorte este store guarda.
    name: &'static str,
    /// O mapa e a política dele.
    cache: MokaCache,
}

impl MokaStore {
    /// Enfileira a escrita, ou a publica direto se não houver escopo.
    ///
    /// Fora de um escopo a escrita vale na hora, e é isso que mantém o boot
    /// funcionando sem cerimônia: `declare_permissions` roda antes de existir
    /// requisição, e exigir um escopo ali só produziria uma moldura vazia.
    async fn write(&self, group: &str, operation: Operation) -> anyhow::Result<()> {
        let write = PendingWrite {
            cache: self.cache.clone(),
            store: self.name,
            group: group.to_owned(),
            operation,
        };

        if !ScopeSlots::is_active() {
            return write.apply().await;
        }

        ScopeSlots::current::<MokaContext>()?.enqueue(write).await;
        Ok(())
    }
}

impl MemoryStore for MokaStore {
    async fn get(&self, group: &str, key: &str) -> anyhow::Result<Option<Arc<Vec<u8>>>> {
        if ScopeSlots::is_active() {
            if let Some(pending) = ScopeSlots::current::<MokaContext>()?
                .lookup(self.name, group, key)
                .await
            {
                return Ok(pending);
            }
        }

        Ok(self
            .cache
            .0
            .get(&(group.to_owned(), key.to_owned()))
            .await
            .map(|(value, _)| value))
    }

    async fn put(
        &self,
        group: &str,
        key: &str,
        value: Vec<u8>,
        ttl: Option<Duration>,
    ) -> anyhow::Result<()> {
        self.write(
            group,
            Operation::Put {
                key: key.to_owned(),
                value: Arc::new(value),
                ttl,
            },
        )
        .await
    }

    async fn clear_group(&self, group: &str) -> anyhow::Result<()> {
        self.write(group, Operation::ClearGroup).await
    }

    async fn keys(&self, group: &str) -> anyhow::Result<Vec<String>> {
        let mut keys: BTreeSet<String> = self
            .cache
            .0
            .iter()
            .filter(|(entry, _)| entry.0 == group)
            .map(|(entry, _)| entry.1.clone())
            .collect();

        if ScopeSlots::is_active() {
            ScopeSlots::current::<MokaContext>()?
                .overlay(self.name, group, &mut keys)
                .await;
        }

        Ok(keys.into_iter().collect())
    }
}
