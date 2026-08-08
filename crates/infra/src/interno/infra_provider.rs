//! A implementação do provider da camada.

use std::sync::Arc;
use std::time::Duration;

use moka::future::Cache;
use sqlx::MySqlPool;

use crate::cache::interno::marker_cache::MarkerCache;
use crate::cache::interno::marker_group_cache::MarkerGroupCache;
use crate::cache::interno::moka_marker_group_repository::MokaMarkerGroupRepository;
use crate::cache::interno::moka_marker_repository::MokaMarkerRepository;
use crate::cache::interno::moka_permission_repository::MokaPermissionRepository;
use crate::cache::interno::moka_read_cache::MokaReadCache;
use crate::cache::interno::permission_cache::PermissionCache;
use crate::cache::interno::read_cache_store::ReadCacheStore;
use crate::cache::ReadCache;
use crate::config::cache_limits::CacheLimits;
use crate::database::interno::mariadb_unit_of_work::MariadbUnitOfWork;
use crate::database::UnitOfWork;
use crate::id::interno::nano_id_generator::NanoIdGenerator;
use crate::id::interno::xid_generator::XidGenerator;
use crate::id::{RandomIdGenerator, SortableIdGenerator};
use crate::logging::interno::tracing_logger_factory::TracingLoggerFactory;
use crate::logging::LoggerFactory;
use crate::provider::InfraProvider;
use crate::query::interno::mariadb_query_factory::MariadbQueryFactory;
use crate::query::interno::mariadb_query_repository::MariadbQueryRepository;
use crate::query::{QueryFactory, QueryRepository};
use crate::repository::mariadb::container_repository::ContainerMariadbRepository;
use crate::repository::mariadb::manifest_repository::ManifestMariadbRepository;
use crate::repository::mariadb::product_repository::ProductMariadbRepository;
use crate::repository::mariadb::role_repository::RoleMariadbRepository;
use crate::repository::mariadb::user_repository::UserMariadbRepository;
use crate::repository::{
    ContainerRepository, ManifestRepository, MarkerGroupRepository, MarkerRepository,
    PermissionRepository, ProductRepository, RoleRepository, UserRepository,
};

/// A implementação do provider. Privada: nenhum crate exporta impl.
pub(crate) struct InfraProviderImpl {
    /// O pool de conexões, compartilhado por clone — é um `Arc` por dentro.
    pool: MySqlPool,
    /// O catálogo de permissões em memória, preenchido no boot.
    permission_cache: PermissionCache,
    /// Os grupos de marcador declarados.
    marker_group_cache: MarkerGroupCache,
    /// Os marcadores em si, com TTL — é onde o refresh token vive.
    marker_cache: MarkerCache,
    /// O cache de leitura, por prefixo de chave.
    read_cache: ReadCacheStore,
}

impl InfraProviderImpl {
    /// Cria os recursos base: o pool já veio pronto, os quatro caches nascem
    /// aqui e são compartilhados por clone pelo resto da vida do processo.
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
    pub(crate) fn new(pool: MySqlPool) -> Self {
        Self {
            pool,
            permission_cache: PermissionCache::new(CacheLimits::METADATA_CACHE_CAPACITY),
            marker_group_cache: MarkerGroupCache::new(CacheLimits::METADATA_CACHE_CAPACITY),
            marker_cache: MarkerCache::new(CacheLimits::MARKER_CACHE_CAPACITY),
            read_cache: Arc::new(
                Cache::builder()
                    .max_capacity(CacheLimits::READ_CACHE_CAPACITY)
                    .time_to_live(Duration::from_secs(CacheLimits::READ_CACHE_TTL_SECONDS))
                    .support_invalidation_closures()
                    .build(),
            ),
        }
    }
}

impl InfraProvider for InfraProviderImpl {
    fn unit_of_work(&self) -> impl UnitOfWork {
        MariadbUnitOfWork::new(self.pool.clone())
    }

    fn user_repository(&self) -> impl UserRepository {
        UserMariadbRepository::new(self.role_repository())
    }

    fn role_repository(&self) -> impl RoleRepository {
        RoleMariadbRepository::new()
    }

    fn product_repository(&self) -> impl ProductRepository {
        ProductMariadbRepository::new()
    }

    fn container_repository(&self) -> impl ContainerRepository {
        ContainerMariadbRepository::new()
    }

    fn manifest_repository(&self) -> impl ManifestRepository {
        ManifestMariadbRepository::new()
    }

    fn permission_repository(&self) -> impl PermissionRepository {
        MokaPermissionRepository::new(self.permission_cache.clone())
    }

    fn marker_group_repository(&self) -> impl MarkerGroupRepository {
        MokaMarkerGroupRepository::new(self.marker_group_cache.clone())
    }

    fn marker_repository(&self) -> impl MarkerRepository {
        MokaMarkerRepository::new(self.marker_cache.clone(), self.marker_group_repository())
    }

    fn query_repository(&self) -> impl QueryRepository {
        MariadbQueryRepository::new()
    }

    fn query_factory(&self) -> impl QueryFactory {
        MariadbQueryFactory::new()
    }

    fn read_cache(&self) -> impl ReadCache {
        MokaReadCache::new(self.read_cache.clone())
    }

    fn random_id_generator(&self) -> impl RandomIdGenerator {
        NanoIdGenerator::new()
    }

    fn sortable_id_generator(&self) -> impl SortableIdGenerator {
        XidGenerator::new()
    }

    fn logger_factory(&self) -> impl LoggerFactory {
        TracingLoggerFactory::new()
    }
}
