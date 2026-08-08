//! Ler pelo cache, e derrubá-lo ao escrever.

use crate::error::AppError;
use portmaster_infra::cache::ReadCache;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::future::Future;

/// O caminho de leitura que passa pelo cache antes de tocar o banco.
pub(crate) struct ReadThrough;

impl ReadThrough {
    /// Serve do cache, ou executa e guarda.
    ///
    /// O valor é guardado como JSON. Não é o formato do fio — quem serializa a
    /// resposta é o `api-http`, com os próprios tipos — é só o jeito de um cache de
    /// bytes guardar um tipo qualquer sem virar um cache por tipo.
    /// Serve do cache, ou calcula e guarda.
    ///
    /// ## Um valor que não desserializa é tratado como ausência
    ///
    /// Ele é de um formato anterior — a View mudou de forma desde que ele foi
    /// guardado. Recalcular em silêncio é o comportamento certo: derrubar a
    /// requisição por causa de um cache velho transformaria um deploy em
    /// incidente.
    ///
    /// ## Falha ao guardar não invalida a resposta
    ///
    /// O cliente já tem o dado correto, e o único prejuízo é o próximo pedido
    /// recalcular.
    pub(crate) async fn cached<C, V, F>(cache: &C, key: &str, load: F) -> Result<V, AppError>
    where
        C: ReadCache,
        V: Serialize + DeserializeOwned,
        F: Future<Output = Result<V, AppError>>,
    {
        if let Some(bytes) = cache.get(key).await? {
            if let Ok(hit) = serde_json::from_slice(&bytes) {
                return Ok(hit);
            }
        }

        let value = load.await?;

        if let Ok(bytes) = serde_json::to_vec(&value) {
            cache.put(key, bytes).await?;
        }

        Ok(value)
    }

    /// Derruba tudo que uma escrita tornou obsoleto.
    ///
    /// Chamada **depois** do commit. Antes, uma leitura concorrente poderia repovoar
    /// o cache com o estado antigo — ainda visível para ela — e a invalidação teria
    /// derrubado exatamente nada.
    pub(crate) async fn invalidate<C: ReadCache>(
        cache: &C,
        prefixes: &[&str],
    ) -> Result<(), AppError> {
        for prefix in prefixes {
            cache.invalidate_prefix(prefix).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::cache_key::CacheKey;
    use crate::cache::invalidation::Invalidation;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use pretty_assertions::assert_eq;

    /// As entradas guardadas pelo cache de teste.
    type Entries = Arc<Mutex<Vec<(String, Vec<u8>)>>>;

    /// Um cache de mentira, que guarda numa tabela e conta as execuções.
    #[derive(Clone, Default)]
    struct FakeCache {
        entries: Entries,
    }

    impl ReadCache for FakeCache {
        async fn get(&self, key: &str) -> anyhow::Result<Option<Arc<Vec<u8>>>> {
            let entries = self.entries.lock().expect("mutex de teste");

            Ok(entries
                .iter()
                .find(|(stored, _)| stored == key)
                .map(|(_, value)| Arc::new(value.clone())))
        }

        async fn put(&self, key: &str, value: Vec<u8>) -> anyhow::Result<()> {
            let mut entries = self.entries.lock().expect("mutex de teste");
            entries.retain(|(stored, _)| stored != key);
            entries.push((key.to_owned(), value));

            Ok(())
        }

        async fn invalidate(&self, key: &str) -> anyhow::Result<()> {
            let mut entries = self.entries.lock().expect("mutex de teste");
            entries.retain(|(stored, _)| stored != key);

            Ok(())
        }

        async fn invalidate_prefix(&self, prefix: &str) -> anyhow::Result<()> {
            let mut entries = self.entries.lock().expect("mutex de teste");
            entries.retain(|(stored, _)| !stored.starts_with(prefix));

            Ok(())
        }
    }

    #[tokio::test]
    async fn o_segundo_pedido_nao_reexecuta() {
        let cache = FakeCache::default();
        let execucoes = Arc::new(AtomicUsize::new(0));

        for _ in 0..2 {
            let contador = Arc::clone(&execucoes);
            let value: i64 = ReadThrough::cached(&cache, "product:list", async move {
                contador.fetch_add(1, Ordering::SeqCst);
                Ok(7)
            })
            .await
            .unwrap();

            assert_eq!(value, 7);
        }

        assert_eq!(execucoes.load(Ordering::SeqCst), 1);
    }

    /// Guardar um erro faria uma indisponibilidade de um segundo durar o TTL
    /// inteiro.
    #[tokio::test]
    async fn a_falha_nao_e_cacheada() {
        let cache = FakeCache::default();

        let result: Result<i64, _> = ReadThrough::cached(&cache, "product:list", async {
            Err(AppError::Unauthenticated)
        })
        .await;

        assert!(result.is_err());
        assert!(cache.get("product:list").await.unwrap().is_none());
    }

    /// A View mudou de forma desde que isto foi guardado.
    ///
    /// Recalcular é o caminho certo; derrubar a requisição transformaria
    /// deploy em incidente.
    #[tokio::test]
    async fn um_valor_de_formato_antigo_e_recalculado() {
        let cache = FakeCache::default();
        cache
            .put("product:list", b"isto nao e um numero".to_vec())
            .await
            .unwrap();

        let value: i64 = ReadThrough::cached(&cache, "product:list", async { Ok(7) })
            .await
            .unwrap();

        assert_eq!(value, 7);
    }

    #[tokio::test]
    async fn a_invalidacao_alcanca_todas_as_variacoes_de_filtro() {
        let cache = FakeCache::default();
        ReadThrough::cached(&cache, "product:list:20::cimento", async { Ok(1_i64) })
            .await
            .unwrap();
        ReadThrough::cached(&cache, "product:list:20::areia", async { Ok(2_i64) })
            .await
            .unwrap();
        ReadThrough::cached(&cache, "role:list:20::", async { Ok(3_i64) })
            .await
            .unwrap();

        ReadThrough::invalidate(&cache, Invalidation::PRODUCT_WRITE)
            .await
            .unwrap();

        assert!(cache
            .get("product:list:20::cimento")
            .await
            .unwrap()
            .is_none());
        assert!(cache.get("product:list:20::areia").await.unwrap().is_none());
        assert!(
            cache.get("role:list:20::").await.unwrap().is_some(),
            "a invalidação de produto não deveria alcançar papéis"
        );
    }

    #[test]
    fn o_filtro_ausente_ocupa_lugar_na_chave() {
        // Sem isso, "sem busca" e "busca vazia" gerariam a mesma chave.
        assert_eq!(
            CacheKey::of(CacheKey::PRODUCT, "list", &["20", "", ""]),
            "product:list:20::"
        );
        assert_eq!(
            CacheKey::of(CacheKey::PRODUCT, "list", &["20", "", "cimento"]),
            "product:list:20::cimento"
        );
    }
}
