//! O cache de leitura em memória.

use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::config::cache_limits::CacheLimits;
use crate::repository::ViewCacheRepository;
use crate::scope::memory::memory_store::MemoryStore;

/// O cache de leitura, sobre o store em memória.
///
/// A chave que ele recebe é a do DQL, crua, sem digest no meio: reduzi-la a um
/// hash só encurtaria o que já é curto, ao preço de uma entrada deixar de dizer
/// a que consulta pertence quando alguém for depurar.
#[derive(Clone)]
pub struct ViewCacheMemoryRepository<S> {
    /// De onde a memória da tarefa vem.
    store: S,
}

impl<S> ViewCacheMemoryRepository<S> {
    /// Monta o repositório.
    pub(crate) const fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: MemoryStore + Send + Sync> ViewCacheRepository for ViewCacheMemoryRepository<S> {
    async fn get<V: DeserializeOwned>(&self, group: &str, key: &str) -> anyhow::Result<Option<V>> {
        let Some(bytes) = self.store.get(group, key).await? else {
            return Ok(None);
        };

        Ok(
            bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
                .ok()
                .map(|(view, _)| view),
        )
    }

    async fn put<V: Serialize + Sync>(
        &self,
        group: &str,
        key: &str,
        view: &V,
    ) -> anyhow::Result<()> {
        let Ok(bytes) = bincode::serde::encode_to_vec(view, bincode::config::standard()) else {
            // Uma View que não serializa não é motivo para derrubar a leitura:
            // ela já está pronta, e o cache é otimização.
            return Ok(());
        };

        self.store
            .put(
                group,
                key,
                bytes,
                Some(Duration::from_secs(CacheLimits::READ_CACHE_TTL_SECONDS)),
            )
            .await
    }

    async fn invalidate(&self, group: &str) -> anyhow::Result<()> {
        self.store.clear_group(group).await
    }
}
