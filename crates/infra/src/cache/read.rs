//! Cache de resultados de leitura.
//!
//! A decisão de **cachear** é do `app` — ele conhece a operação e sabe se o dado
//! tolera estar um pouco velho. O **TTL** é daqui: concentrar a política de
//! expiração num lugar só evita que cada caso de uso invente o seu prazo.
//!
//! A ordem no caso de uso importa e não é acidental: autoriza **antes** de
//! consultar o cache. Um cache consultado antes da checagem de permissão
//! entregaria dado a quem não pode vê-lo — o cache não sabe quem está
//! perguntando.

use moka::future::Cache;
use std::sync::Arc;

/// O que o cache guarda: a resposta já serializada, sob a chave da consulta.
///
/// Bytes e não a `View` tipada: uma `View` por tipo exigiria um cache por tipo,
/// e o que se quer é um só, agnóstico do que está passando por ele.
pub(crate) type ReadCacheStore = Arc<Cache<String, Arc<Vec<u8>>>>;

/// Cache de leitura, indiferente ao que guarda.
#[trait_variant::make(Send)]
pub trait ReadCache {
    /// O valor sob a chave, se ainda válido.
    async fn get(&self, key: &str) -> anyhow::Result<Option<Arc<Vec<u8>>>>;

    /// Guarda o valor sob a chave, com o TTL desta camada.
    async fn put(&self, key: &str, value: Vec<u8>) -> anyhow::Result<()>;

    /// Descarta uma chave.
    async fn invalidate(&self, key: &str) -> anyhow::Result<()>;

    /// Descarta tudo que começa com um prefixo.
    ///
    /// É o que uma escrita usa: alterar um produto invalida todas as listagens
    /// de produto de uma vez, sem que o caso de uso precise enumerar quais
    /// combinações de filtro existem por aí.
    async fn invalidate_prefix(&self, prefix: &str) -> anyhow::Result<()>;
}

/// A implementação sobre Moka.
pub(crate) struct MokaReadCache {
    store: ReadCacheStore,
}

impl MokaReadCache {
    /// Monta o cache sobre o store do processo.
    pub(crate) fn new(store: ReadCacheStore) -> Self {
        Self { store }
    }
}

impl ReadCache for MokaReadCache {
    async fn get(&self, key: &str) -> anyhow::Result<Option<Arc<Vec<u8>>>> {
        Ok(self.store.get(key).await)
    }

    async fn put(&self, key: &str, value: Vec<u8>) -> anyhow::Result<()> {
        // O TTL é uniforme aqui, então vem da política do cache (montada no
        // `register`) em vez de viajar com cada entrada.
        self.store.insert(key.to_owned(), Arc::new(value)).await;
        Ok(())
    }

    async fn invalidate(&self, key: &str) -> anyhow::Result<()> {
        self.store.invalidate(key).await;
        Ok(())
    }

    async fn invalidate_prefix(&self, prefix: &str) -> anyhow::Result<()> {
        // O Moka não indexa por prefixo, então isto varre as chaves vivas. É
        // aceitável porque roda depois de uma escrita — que é rara comparada à
        // leitura — e o cache é limitado por capacidade.
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
    use pretty_assertions::assert_eq;

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

    #[tokio::test]
    async fn invalidar_por_prefixo_pega_todas_as_variacoes_de_filtro() {
        // O caso real: alterar um produto precisa derrubar toda listagem de
        // produto, sem que o caso de uso saiba quais filtros foram usados.
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
