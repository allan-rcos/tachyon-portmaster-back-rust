//! Persistência de grupos de marcador em memória.

use portmaster_domain::domain::MarkerGroup;

use crate::repository::MarkerGroupRepository;
use crate::scope::memory::memory_store::MemoryStore;

/// O grupo deste repositório — o que num repositório do `MariaDB` seria a tabela.
const GROUP: &str = "marker-group";

/// O repositório de grupos de marcador.
///
/// Vive num store próprio, e não no mesmo das permissões: se dividissem o
/// armazenamento, `PermissionRepository::all()` devolveria `refresh-token` junto
/// das permissões, e o papel que o `POST /setup` cria — que recebe tudo que
/// estiver registrado — ganharia uma concessão que ninguém declarou. O schema
/// anterior os separava em duas tabelas pelo mesmo motivo.
#[derive(Clone)]
pub struct MarkerGroupMemoryRepository<S> {
    /// De onde a memória da tarefa vem.
    store: S,
}

impl<S> MarkerGroupMemoryRepository<S> {
    /// Monta o repositório.
    pub(crate) const fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: MemoryStore + Send + Sync> MarkerGroupRepository for MarkerGroupMemoryRepository<S> {
    async fn register(&self, group: &dyn MarkerGroup) -> anyhow::Result<()> {
        self.store.put(GROUP, group.slug(), Vec::new(), None).await
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

    /// Grupo mínimo, sem passar pelo `TableModule`.
    struct StubGroup(&'static str);
    impl MarkerGroup for StubGroup {
        fn slug(&self) -> &str {
            self.0
        }
    }

    fn repository() -> MarkerGroupMemoryRepository<MokaStore> {
        MarkerGroupMemoryRepository::new(MokaStore::new("marker-group", MokaCache::new(100)))
    }

    #[tokio::test]
    async fn registrar_e_consultar() {
        let repository = repository();
        repository
            .register(&StubGroup("refresh-token"))
            .await
            .unwrap();

        assert!(repository.has("refresh-token").await.unwrap());
        assert!(!repository.has("nunca-declarado").await.unwrap());
    }

    #[tokio::test]
    async fn registrar_duas_vezes_nao_e_erro() {
        let repository = repository();
        repository
            .register(&StubGroup("refresh-token"))
            .await
            .unwrap();
        repository
            .register(&StubGroup("refresh-token"))
            .await
            .unwrap();

        assert!(repository.has("refresh-token").await.unwrap());
    }
}
