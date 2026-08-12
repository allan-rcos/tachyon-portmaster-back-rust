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
#[path = "tests/permission_repository_test.rs"]
mod tests;
