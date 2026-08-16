//! Quem serve os repositórios em memória.

use crate::repository::memory::marker_group_repository::marker_group_repository;
use crate::repository::memory::marker_repository::marker_repository;
use crate::repository::memory::permission_repository::permission_repository;
use crate::repository::memory::view_cache_repository::view_cache_repository;
use crate::repository::{
    MarkerGroupRepository, MarkerRepository, PermissionRepository, ViewCacheRepository,
};
use crate::scope::ScopeProvider;

/// Os repositórios cujo armazenamento é a memória do processo.
///
/// Infalíveis, ao contrário dos do `MariaDB`: um mapa em memória não depende de
/// segredo nenhum para existir, então a assinatura não pede um `?` a quem não
/// pode falhar.
pub(crate) struct MemoryRepositoryProvider;

impl MemoryRepositoryProvider {
    /// O catálogo de permissões.
    pub(crate) fn permission() -> impl PermissionRepository + Sync + Clone + use<> + 'static {
        permission_repository(ScopeProvider::permissions())
    }

    /// O registro de grupos de marcador.
    pub(crate) fn marker_group() -> impl MarkerGroupRepository + Sync + Clone + use<> + 'static {
        marker_group_repository(ScopeProvider::marker_groups())
    }

    /// Os marcadores, já ligados ao registro de grupos que os valida.
    pub(crate) fn marker() -> impl MarkerRepository + Sync + Clone + use<> + 'static {
        marker_repository(ScopeProvider::markers(), Self::marker_group())
    }

    /// O cache do lado de leitura.
    pub(crate) fn view_cache() -> impl ViewCacheRepository + Sync + Clone + use<> + 'static {
        view_cache_repository(ScopeProvider::views())
    }
}
