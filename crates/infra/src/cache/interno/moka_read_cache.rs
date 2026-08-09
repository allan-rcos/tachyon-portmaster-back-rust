//! O cache de leitura sobre o moka.

use std::sync::Arc;

use crate::cache::interno::read_cache_store::ReadCacheStore;
use crate::cache::ReadCache;

/// A implementação sobre Moka.
#[derive(Clone)]
pub(crate) struct MokaReadCache {
    /// As entradas de leitura, chaveadas por prefixo e argumento.
    store: ReadCacheStore,
}

impl MokaReadCache {
    /// Monta o cache sobre o store do processo.
    pub(crate) const fn new(store: ReadCacheStore) -> Self {
        Self { store }
    }
}

impl ReadCache for MokaReadCache {
    async fn get(&self, key: &str) -> anyhow::Result<Option<Arc<Vec<u8>>>> {
        Ok(self.store.get(key).await)
    }

    /// O TTL é uniforme aqui, então vem da política do cache (montada no
    /// `register`) em vez de viajar com cada entrada.
    async fn put(&self, key: &str, value: Vec<u8>) -> anyhow::Result<()> {
        self.store.insert(key.to_owned(), Arc::new(value)).await;
        Ok(())
    }

    async fn invalidate(&self, key: &str) -> anyhow::Result<()> {
        self.store.invalidate(key).await;
        Ok(())
    }

    /// O Moka não indexa por prefixo, então isto varre as chaves vivas.
    ///
    /// É aceitável porque roda depois de uma escrita — que é rara comparada à
    /// leitura — e o cache é limitado por capacidade.
    async fn invalidate_prefix(&self, prefix: &str) -> anyhow::Result<()> {
        let prefix = prefix.to_owned();
        self.store
            .invalidate_entries_if(move |key, _| key.starts_with(&prefix))
            .map_err(|e| anyhow::anyhow!("falha ao invalidar o cache por prefixo: {e}"))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use moka::future::Cache;
    use pretty_assertions::assert_eq;
    use std::sync::Arc;

    fn cache() -> MokaReadCache {
        MokaReadCache::new(Arc::new(
            Cache::builder()
                .max_capacity(100)
                .time_to_live(std::time::Duration::from_secs(60))
                .support_invalidation_closures()
                .build(),
        ))
    }

    #[tokio::test]
    async fn guarda_e_devolve() {
        let cache = cache();
        cache
            .put("products:page=1", b"payload".to_vec())
            .await
            .unwrap();

        let hit = cache.get("products:page=1").await.unwrap();
        assert_eq!(
            hit.as_deref().map(Vec::as_slice),
            Some(b"payload".as_slice())
        );
    }

    #[tokio::test]
    async fn chave_ausente_nao_e_erro() {
        // Miss é o caminho normal, não uma falha.
        assert!(cache().get("nunca-visto").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn invalidar_descarta_a_chave() {
        let cache = cache();
        cache
            .put("products:page=1", b"payload".to_vec())
            .await
            .unwrap();
        cache.invalidate("products:page=1").await.unwrap();

        assert!(cache.get("products:page=1").await.unwrap().is_none());
    }

    /// O caso real: alterar um produto precisa derrubar toda listagem de
    /// produto, sem que o caso de uso saiba quais filtros foram usados.
    #[tokio::test]
    async fn invalidar_por_prefixo_pega_todas_as_variacoes_de_filtro() {
        let cache = cache();
        cache.put("products:page=1", b"a".to_vec()).await.unwrap();
        cache.put("products:page=2", b"b".to_vec()).await.unwrap();
        cache.put("containers:page=1", b"c".to_vec()).await.unwrap();

        cache.invalidate_prefix("products:").await.unwrap();
        cache.store.run_pending_tasks().await;

        assert!(cache.get("products:page=1").await.unwrap().is_none());
        assert!(cache.get("products:page=2").await.unwrap().is_none());
        assert!(
            cache.get("containers:page=1").await.unwrap().is_some(),
            "a invalidação não deveria passar do prefixo pedido"
        );
    }
}
