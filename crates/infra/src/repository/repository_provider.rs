//! Quem serve os repositórios, de qualquer armazenamento.

use crate::repository::mariadb::MariaDbRepositoryProvider;
use crate::repository::memory::MemoryRepositoryProvider;
use crate::repository::{
    ContainerRepository, ManifestRepository, MarkerGroupRepository, MarkerRepository,
    PermissionRepository, ProductRepository, RoleRepository, UserRepository, ViewCacheRepository,
};

/// Os repositórios, sem dizer de onde eles leem.
///
/// Encapsula os dois providers de baixo, e é a fronteira em que o armazenamento
/// deixa de aparecer: quem pede um `PermissionRepository` não fica sabendo que
/// ele mora em RAM, e quem pede um `UserRepository` não fica sabendo que ele
/// fala SQL. Trocar um pelo outro é mexer no provider de baixo, e em nada mais.
pub(crate) struct RepositoryProvider;

impl RepositoryProvider {
    /// A persistência de usuários.
    pub(crate) fn user() -> anyhow::Result<impl UserRepository + Sync + Clone + use<> + 'static> {
        MariaDbRepositoryProvider::user()
    }

    /// A persistência de papéis.
    pub(crate) fn role() -> anyhow::Result<impl RoleRepository + Sync + Clone + use<> + 'static> {
        MariaDbRepositoryProvider::role()
    }

    /// A persistência de produtos.
    pub(crate) fn product(
    ) -> anyhow::Result<impl ProductRepository + Sync + Clone + use<> + 'static> {
        MariaDbRepositoryProvider::product()
    }

    /// A persistência de contêineres.
    pub(crate) fn container(
    ) -> anyhow::Result<impl ContainerRepository + Sync + Clone + use<> + 'static> {
        MariaDbRepositoryProvider::container()
    }

    /// A persistência de manifesto.
    pub(crate) fn manifest(
    ) -> anyhow::Result<impl ManifestRepository + Sync + Clone + use<> + 'static> {
        MariaDbRepositoryProvider::manifest()
    }

    /// O catálogo de permissões.
    pub(crate) fn permission() -> impl PermissionRepository + Sync + Clone + use<> + 'static {
        MemoryRepositoryProvider::permission()
    }

    /// O registro de grupos de marcador.
    pub(crate) fn marker_group() -> impl MarkerGroupRepository + Sync + Clone + use<> + 'static {
        MemoryRepositoryProvider::marker_group()
    }

    /// Os marcadores.
    pub(crate) fn marker() -> impl MarkerRepository + Sync + Clone + use<> + 'static {
        MemoryRepositoryProvider::marker()
    }

    /// O cache do lado de leitura.
    pub(crate) fn view_cache() -> impl ViewCacheRepository + Sync + Clone + use<> + 'static {
        MemoryRepositoryProvider::view_cache()
    }
}
