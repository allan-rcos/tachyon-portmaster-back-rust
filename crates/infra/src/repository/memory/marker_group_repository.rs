//! Persistência de grupos de marcador em memória.

use portmaster_domain::domain::MarkerGroup;

use crate::repository::MarkerGroupRepository;
use crate::scope::memory::memory_store::MemoryStore;

/// O grupo deste repositório — o que num repositório do `MariaDB` seria a tabela.
const GROUP: &str = "marker-group";

/// Monta o repositório de grupos de marcador.
///
/// Recebe um store próprio, e não o mesmo das permissões: se dividissem o
/// armazenamento, `PermissionRepository::all()` devolveria `refresh-token` junto
/// das permissões, e o papel que o `POST /setup` cria — que recebe tudo que
/// estiver registrado — ganharia uma concessão que ninguém declarou. O schema
/// anterior os separava em duas tabelas pelo mesmo motivo.
pub(super) fn marker_group_repository<S>(
    store: S,
) -> impl MarkerGroupRepository + Sync + Clone + use<S> + 'static
where
    S: MemoryStore + Send + Sync + Clone + 'static,
{
    MarkerGroupMemoryRepository { store }
}

/// O repositório, sobre o store em memória.
#[derive(Clone)]
struct MarkerGroupMemoryRepository<S> {
    /// De onde a memória da tarefa vem.
    store: S,
}

impl<S: MemoryStore + Send + Sync> MarkerGroupRepository for MarkerGroupMemoryRepository<S> {
    async fn register(&self, group: &dyn MarkerGroup) -> anyhow::Result<()> {
        self.store.put(GROUP, group.slug(), Vec::new(), None).await
    }

    async fn has(&self, slug: &str) -> anyhow::Result<bool> {
        Ok(self.store.get(GROUP, slug).await?.is_some())
    }
}
