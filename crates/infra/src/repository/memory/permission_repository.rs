//! Persistência de permissões em memória.

use portmaster_domain::domain::Permission;

use crate::repository::PermissionRepository;
use crate::scope::memory::memory_store::MemoryStore;

/// O grupo deste repositório — o que num repositório do `MariaDB` seria a tabela.
const GROUP: &str = "permission";

/// O repositório de permissões.
#[derive(Clone)]
pub struct PermissionMemoryRepository<S> {
    /// De onde a memória da tarefa vem.
    store: S,
}

impl<S> PermissionMemoryRepository<S> {
    /// Monta o repositório.
    pub(crate) const fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: MemoryStore + Send + Sync> PermissionRepository for PermissionMemoryRepository<S> {
    /// Idempotente por slug: cada caso de uso declara a sua permissão ao ser
    /// construído, e nada garante que isso aconteça uma vez só.
    ///
    /// O valor é vazio porque a identidade **é** o slug. Um repositório do
    /// `MariaDB` gravaria uma linha com uma coluna só pelo mesmo motivo.
    ///
    /// Sem prazo: o catálogo é preenchido no boot e precisa valer enquanto o
    /// processo valer. Uma permissão que expirasse recusaria em silêncio quem
    /// tinha direito.
    async fn register(&self, permission: &dyn Permission) -> anyhow::Result<()> {
        self.store
            .put(GROUP, permission.slug(), Vec::new(), None)
            .await
    }

    async fn all(&self) -> anyhow::Result<Vec<String>> {
        self.store.keys(GROUP).await
    }

    async fn has(&self, slug: &str) -> anyhow::Result<bool> {
        Ok(self.store.get(GROUP, slug).await?.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scope::memory::intern::moka_cache::MokaCache;
    use crate::scope::memory::intern::moka_store::MokaStore;
    use pretty_assertions::assert_eq;

    /// Permissão mínima, sem passar pelo `TableModule`.
    struct StubPermission(&'static str);
    impl Permission for StubPermission {
        fn slug(&self) -> &str {
            self.0
        }
    }

    fn repository() -> PermissionMemoryRepository<MokaStore> {
        PermissionMemoryRepository::new(MokaStore::new("permission", MokaCache::new(100)))
    }

    /// Cada caso de uso declara a sua permissão ao ser construído, e o provider
    /// pode construí-lo mais de uma vez.
    #[tokio::test]
    async fn registrar_a_mesma_permissao_duas_vezes_nao_duplica() {
        let repository = repository();

        repository
            .register(&StubPermission("product:create"))
            .await
            .unwrap();
        repository
            .register(&StubPermission("product:create"))
            .await
            .unwrap();

        assert_eq!(repository.all().await.unwrap(), vec!["product:create"]);
    }

    #[tokio::test]
    async fn a_listagem_sai_ordenada() {
        let repository = repository();

        for slug in ["user:list", "container:seal", "product:create"] {
            repository.register(&StubPermission(slug)).await.unwrap();
        }

        assert_eq!(
            repository.all().await.unwrap(),
            vec!["container:seal", "product:create", "user:list"]
        );
    }

    #[tokio::test]
    async fn has_responde_pelo_que_foi_registrado() {
        let repository = repository();
        repository
            .register(&StubPermission("metrics:read"))
            .await
            .unwrap();

        assert!(repository.has("metrics:read").await.unwrap());
        assert!(!repository.has("metrics:write").await.unwrap());
    }

    /// O boot roda fora de qualquer escopo, e precisa continuar valendo na hora.
    #[tokio::test]
    async fn escrita_fora_do_escopo_vale_na_hora() {
        let repository = repository();
        repository
            .register(&StubPermission("metrics:read"))
            .await
            .unwrap();

        assert!(!crate::scope::MasterScope::is_active());
        assert!(repository.has("metrics:read").await.unwrap());
    }
}
