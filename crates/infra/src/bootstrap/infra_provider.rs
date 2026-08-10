//! A implementação do provider da camada.

use crate::bootstrap::provider::InfraProvider;
use crate::config::cache_limits::CacheLimits;
use crate::logging::intern::tracing_logger_factory::TracingLoggerFactory;
use crate::logging::LoggerFactory;
use crate::query::intern::mariadb_query_repository::MariadbQueryRepository;
use crate::query::QueryRepository;
use crate::repository::mariadb::container_repository::ContainerMariadbRepository;
use crate::repository::mariadb::manifest_repository::ManifestMariadbRepository;
use crate::repository::mariadb::product_repository::ProductMariadbRepository;
use crate::repository::mariadb::role_repository::RoleMariadbRepository;
use crate::repository::mariadb::user_repository::UserMariadbRepository;
use crate::repository::memory::marker_group_repository::MarkerGroupMemoryRepository;
use crate::repository::memory::marker_repository::MarkerMemoryRepository;
use crate::repository::memory::permission_repository::PermissionMemoryRepository;
use crate::repository::memory::view_cache_repository::ViewCacheMemoryRepository;
use crate::repository::{
    ContainerRepository, ManifestRepository, MarkerGroupRepository, MarkerRepository,
    PermissionRepository, ProductRepository, RoleRepository, UserRepository, ViewCacheRepository,
};
use crate::scope::database::intern::mariadb_unit_of_work::MariaDbUnitOfWork;
use crate::scope::memory::intern::moka_cache::MokaCache;
use crate::scope::memory::intern::moka_store::MokaStore;

/// A implementação do provider. Privada: nenhum crate exporta impl.
pub(crate) struct InfraProviderImpl {
    /// O acesso ao banco, compartilhado por clone — o pool é um `Arc` por dentro.
    database: MariaDbUnitOfWork,
    /// O catálogo de permissões em memória, preenchido no boot.
    permissions: MokaCache,
    /// Os grupos de marcador declarados.
    marker_groups: MokaCache,
    /// Os marcadores em si, com TTL — é onde o refresh token vive.
    markers: MokaCache,
    /// O cache de leitura, por grupo.
    views: MokaCache,
}

impl InfraProviderImpl {
    /// Cria os recursos base: o acesso ao banco já veio pronto, os quatro caches
    /// nascem aqui e são compartilhados por clone pelo resto da vida do
    /// processo.
    ///
    /// ## Por que permissão e grupo de marcador têm mapas separados
    ///
    /// São vocabulários diferentes. Compartilhar um mapa só faria o grupo
    /// `refresh-token` aparecer na listagem de permissões — e, pior, ser
    /// concedido ao papel que o `POST /setup` cria, que recebe tudo que
    /// estiver registrado. O schema anterior os separava em duas tabelas pelo
    /// mesmo motivo.
    ///
    /// ## Por que o cache de leitura liga os closures de invalidação
    ///
    /// A invalidação por prefixo depende deles: sem os closures, uma escrita
    /// não teria como derrubar todas as listagens afetadas.
    pub(crate) fn new(database: MariaDbUnitOfWork) -> Self {
        Self {
            database,
            permissions: MokaCache::new(CacheLimits::METADATA_CACHE_CAPACITY),
            marker_groups: MokaCache::new(CacheLimits::METADATA_CACHE_CAPACITY),
            markers: MokaCache::new(CacheLimits::MARKER_CACHE_CAPACITY),
            views: MokaCache::new(CacheLimits::READ_CACHE_CAPACITY),
        }
    }
}

impl InfraProvider for InfraProviderImpl {
    fn user_repository(&self) -> impl UserRepository + Clone + use<> + 'static {
        UserMariadbRepository::new(self.role_repository(), self.database.clone())
    }

    fn role_repository(&self) -> impl RoleRepository + Clone + use<> + 'static {
        RoleMariadbRepository::new(self.database.clone())
    }

    fn product_repository(&self) -> impl ProductRepository + Clone + use<> + 'static {
        ProductMariadbRepository::new(self.database.clone())
    }

    fn container_repository(&self) -> impl ContainerRepository + Clone + use<> + 'static {
        ContainerMariadbRepository::new(self.database.clone())
    }

    fn manifest_repository(&self) -> impl ManifestRepository + Clone + use<> + 'static {
        ManifestMariadbRepository::new(self.database.clone())
    }

    fn permission_repository(&self) -> impl PermissionRepository + Clone + use<> + 'static {
        PermissionMemoryRepository::new(MokaStore::new("permission", self.permissions.clone()))
    }

    fn marker_group_repository(&self) -> impl MarkerGroupRepository + Clone + use<> + 'static {
        MarkerGroupMemoryRepository::new(MokaStore::new("marker-group", self.marker_groups.clone()))
    }

    fn marker_repository(&self) -> impl MarkerRepository + Clone + use<> + 'static {
        MarkerMemoryRepository::new(
            MokaStore::new("marker", self.markers.clone()),
            self.marker_group_repository(),
        )
    }

    fn query_repository(&self) -> impl QueryRepository + Clone + use<> + 'static {
        MariadbQueryRepository::new(self.database.clone())
    }

    fn view_cache_repository(&self) -> impl ViewCacheRepository + Clone + use<> + 'static {
        ViewCacheMemoryRepository::new(MokaStore::new("view", self.views.clone()))
    }

    fn logger_factory(&self) -> impl LoggerFactory + use<> + 'static {
        TracingLoggerFactory::new()
    }
}
