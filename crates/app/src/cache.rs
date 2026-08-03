//! O cache de leitura, do ponto de vista de quem decide usá-lo.
//!
//! **Cachear é decisão do `app`**: só ele sabe se a operação tolera um dado um
//! pouco velho. **Quanto tempo** é decisão da `infra`, que concentra o TTL num
//! lugar só — se cada caso de uso inventasse o seu prazo, a política de
//! expiração do sistema seria a soma de 38 opiniões.
//!
//! ## A ordem importa: autorizar antes de consultar
//!
//! O cache é indexado pela consulta, não por quem pergunta. Consultá-lo antes de
//! verificar a permissão entregaria a resposta a quem não pode vê-la — e o
//! acerto de cache é justamente o caminho que não passa pelo banco, onde nada
//! mais restaria para barrar o acesso. Por isso [`cached`] nunca autoriza: quem
//! a chama já autorizou.
//!
//! ## Invalidação por prefixo
//!
//! Uma escrita não sabe quais combinações de filtro existem no cache — quantas
//! páginas de produto foram pedidas, com quais buscas. Então ela derruba o
//! **prefixo** inteiro. Grosseiro de propósito: o custo é recalcular leituras
//! que ainda valiam, e o custo do contrário é servir dado que já não existe.

use std::future::Future;

use portmaster_infra::cache::ReadCache;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::AppError;

/// Os prefixos de chave, um por recurso.
pub(crate) mod prefix {
    /// Leituras de conta — o próprio usuário e seus papéis.
    pub(crate) const ACCOUNT: &str = "account:";
    /// Leituras de contêiner, inclusive o resumo com carga e telemetria.
    pub(crate) const CONTAINER: &str = "container:";
    /// O painel do pátio.
    pub(crate) const METRICS: &str = "metrics:";
    /// Leituras de produto.
    pub(crate) const PRODUCT: &str = "product:";
    /// Leituras de papel.
    pub(crate) const ROLE: &str = "role:";
    /// Leituras de usuário.
    pub(crate) const USER: &str = "user:";
}

/// O que uma escrita de produto torna obsoleto.
///
/// O painel entra junto porque conta `registered_products`: cadastrar um produto
/// muda um número que nada em `product:` alcança.
pub(crate) const PRODUCT_WRITE: &[&str] = &[prefix::PRODUCT, prefix::METRICS];

/// O que uma escrita de contêiner torna obsoleto.
pub(crate) const CONTAINER_WRITE: &[&str] = &[prefix::CONTAINER, prefix::METRICS];

/// O que um embarque ou desembarque torna obsoleto.
///
/// O contêiner muda de peso e de status, o resumo muda de carga e de telemetria,
/// e o painel muda a carga do pátio — três leituras diferentes atingidas por uma
/// operação só.
pub(crate) const MANIFEST_WRITE: &[&str] = &[prefix::CONTAINER, prefix::METRICS];

/// O que uma escrita de usuário torna obsoleto.
///
/// `account:` porque a conta é a mesma pessoa vista de outro ângulo, e `role:`
/// porque a listagem de papéis carrega `user_count`.
pub(crate) const USER_WRITE: &[&str] = &[prefix::USER, prefix::ACCOUNT, prefix::ROLE];

/// O que uma escrita de papel torna obsoleto.
///
/// `user:` e `account:` porque os dois trazem os papéis aninhados: trocar as
/// permissões de um papel muda toda conta que o carrega.
pub(crate) const ROLE_WRITE: &[&str] = &[prefix::ROLE, prefix::USER, prefix::ACCOUNT];

/// Serve do cache, ou executa e guarda.
///
/// O valor é guardado como JSON. Não é o formato do fio — quem serializa a
/// resposta é o `api-http`, com os próprios tipos — é só o jeito de um cache de
/// bytes guardar um tipo qualquer sem virar um cache por tipo.
pub(crate) async fn cached<C, V, F>(cache: &C, key: &str, load: F) -> Result<V, AppError>
where
    C: ReadCache,
    V: Serialize + DeserializeOwned,
    F: Future<Output = Result<V, AppError>>,
{
    if let Some(bytes) = cache.get(key).await? {
        // Um valor que não desserializa é de um formato anterior — a View mudou
        // de forma desde que ele foi guardado. Tratar como ausência recalcula em
        // silêncio, que é o comportamento certo: derrubar a requisição por causa
        // de um cache velho transformaria um deploy em incidente.
        if let Ok(hit) = serde_json::from_slice(&bytes) {
            return Ok(hit);
        }
    }

    let value = load.await?;

    // Falha ao serializar não invalida a resposta: o cliente já tem o dado
    // correto, e o único prejuízo é o próximo pedido recalcular.
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
pub(crate) async fn invalidate<C: ReadCache>(cache: &C, prefixes: &[&str]) -> Result<(), AppError> {
    for prefix in prefixes {
        cache.invalidate_prefix(prefix).await?;
    }

    Ok(())
}

/// Monta a chave de uma leitura a partir dos seus parâmetros.
///
/// Todo parâmetro entra, mesmo ausente (como string vazia): uma chave que omite
/// o filtro nulo faria "sem busca" e "busca vazia" colidirem, e a segunda
/// receberia a resposta da primeira.
pub(crate) fn key(prefix: &str, operation: &str, parts: &[&str]) -> String {
    let mut key = String::with_capacity(prefix.len() + operation.len() + 16);

    key.push_str(prefix);
    key.push_str(operation);

    for part in parts {
        key.push(':');
        key.push_str(part);
    }

    key
}

#[cfg(test)]
mod tests {
    use super::*;
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
            let value: i64 = cached(&cache, "product:list", async move {
                contador.fetch_add(1, Ordering::SeqCst);
                Ok(7)
            })
            .await
            .unwrap();

            assert_eq!(value, 7);
        }

        assert_eq!(execucoes.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a_falha_nao_e_cacheada() {
        // Guardar um erro faria uma indisponibilidade de um segundo durar o TTL
        // inteiro.
        let cache = FakeCache::default();

        let result: Result<i64, _> = cached(&cache, "product:list", async {
            Err(AppError::Unauthenticated)
        })
        .await;

        assert!(result.is_err());
        assert!(cache.get("product:list").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn um_valor_de_formato_antigo_e_recalculado() {
        // A View mudou de forma desde que isto foi guardado. Recalcular é o
        // caminho certo; derrubar a requisição transformaria deploy em
        // incidente.
        let cache = FakeCache::default();
        cache
            .put("product:list", b"isto nao e um numero".to_vec())
            .await
            .unwrap();

        let value: i64 = cached(&cache, "product:list", async { Ok(7) })
            .await
            .unwrap();

        assert_eq!(value, 7);
    }

    #[tokio::test]
    async fn a_invalidacao_alcanca_todas_as_variacoes_de_filtro() {
        let cache = FakeCache::default();
        cached(&cache, "product:list:20::cimento", async { Ok(1_i64) })
            .await
            .unwrap();
        cached(&cache, "product:list:20::areia", async { Ok(2_i64) })
            .await
            .unwrap();
        cached(&cache, "role:list:20::", async { Ok(3_i64) })
            .await
            .unwrap();

        invalidate(&cache, PRODUCT_WRITE).await.unwrap();

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
            key(prefix::PRODUCT, "list", &["20", "", ""]),
            "product:list:20::"
        );
        assert_eq!(
            key(prefix::PRODUCT, "list", &["20", "", "cimento"]),
            "product:list:20::cimento"
        );
    }
}
