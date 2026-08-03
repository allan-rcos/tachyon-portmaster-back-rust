//! # portmaster-infra
//!
//! Adaptadores de I/O. Depende só do `domain`, e só para pegar os traits de
//! objeto de domínio e implementá-los nos seus entities.
//!
//! É dona dos **ports de I/O** — repositórios, `UnitOfWork`, cache, `Logger`.
//! Eles ficam aqui e não no `domain` porque quem os consome é o `app`, que
//! conhece esta camada; o `domain` não sabe que existe banco.
//!
//! ## O `i64` para de existir aqui
//!
//! Esta é a única camada que enxerga o inteiro do Snowflake. Ao ler uma linha
//! deriva o base62 que o trait expõe; ao gravar, decodifica de volta para
//! `BIGINT`. Se o id físico mudar um dia, só este mapeamento muda.
//!
//! ## O que não mora aqui
//!
//! **JWT.** É assunto exclusivo do `api-http`: nem esta camada nem o `app` sabem
//! o que é um token. O que existe aqui é a primitiva de marcador — um booleano
//! com prazo — e é a apresentação que decide chamar aquilo de sessão.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod cache;
pub mod config;
pub mod database;
pub mod id;
pub mod logging;
pub mod query;
pub mod repository;

pub(crate) mod entity;
pub(crate) mod text;

use std::sync::Arc;
use std::time::Duration;

use moka::future::Cache;
use sqlx::MySqlPool;

use cache::marker::{build_cache as build_marker_cache, MarkerCache, MokaMarkerRepository};
use cache::metadata::{
    MarkerGroupCache, MokaMarkerGroupRepository, MokaPermissionRepository, PermissionCache,
};
use cache::read::{MokaReadCache, ReadCacheStore};
use config::{
    InfraSecrets, MARKER_CACHE_CAPACITY, METADATA_CACHE_CAPACITY, READ_CACHE_CAPACITY,
    READ_CACHE_TTL_SECONDS,
};
use database::uow::MariadbUnitOfWork;
use id::{NanoIdGenerator, RandomIdGenerator, SortableIdGenerator, XidGenerator};
use logging::{LoggerFactory, TracingLoggerFactory};
use query::{MariadbQueryFactory, MariadbQueryRepository};
use repository::mariadb::container::ContainerMariadbRepository;
use repository::mariadb::manifest::ManifestMariadbRepository;
use repository::mariadb::product::ProductMariadbRepository;
use repository::mariadb::role::RoleMariadbRepository;
use repository::mariadb::user::UserMariadbRepository;
use repository::{
    ContainerRepository, ManifestRepository, MarkerGroupRepository, MarkerRepository,
    PermissionRepository, ProductRepository, RoleRepository, UserRepository,
};

/// Os factories da `infra`.
///
/// Cada método devolve `impl Trait` — contrato, nunca tipo concreto. Os
/// **recursos base** (pool, caches) nascem uma vez no [`register`] e são
/// entregues por clone barato; os **serviços** são reconstruídos a cada chamada,
/// porque são structs de campos leves e custam praticamente nada.
pub trait InfraProvider {
    /// A unidade de trabalho da requisição.
    fn unit_of_work(&self) -> impl database::UnitOfWork + Sync;

    /// Persistência de usuários.
    fn user_repository(&self) -> impl UserRepository + Sync;

    /// Persistência de papéis.
    fn role_repository(&self) -> impl RoleRepository + Sync;

    /// Persistência de produtos.
    fn product_repository(&self) -> impl ProductRepository + Sync;

    /// Persistência de contêineres.
    fn container_repository(&self) -> impl ContainerRepository + Sync;

    /// Persistência de carga e telemetria.
    fn manifest_repository(&self) -> impl ManifestRepository + Sync;

    /// Registro de permissões.
    fn permission_repository(&self) -> impl PermissionRepository + Sync;

    /// Registro de grupos de marcador.
    fn marker_group_repository(&self) -> impl MarkerGroupRepository + Sync;

    /// Marcadores booleanos com prazo.
    fn marker_repository(&self) -> impl MarkerRepository + Sync;

    /// A fonte única do lado de leitura.
    ///
    /// Roda qualquer consulta que a [`query::QueryFactory`] souber descrever, e
    /// nada além delas.
    fn query_repository(&self) -> impl query::QueryRepository + Sync;

    /// Os descritores de consulta que o `app` pode pedir.
    ///
    /// Separada do repositório porque as duas responsabilidades são distintas:
    /// esta declara **o que** dá para consultar, aquela sabe **executar**. Fundir
    /// as duas num objeto só faria cada consulta nova mexer no executor.
    fn query_factory(&self) -> impl query::QueryFactory + Send + Sync;

    /// Cache de leitura.
    fn read_cache(&self) -> impl cache::ReadCache + Sync;

    /// Gerador de id opaco, para o refresh token.
    fn random_id_generator(&self) -> impl RandomIdGenerator;

    /// Gerador de id ordenável, para o `request_id`.
    fn sortable_id_generator(&self) -> impl SortableIdGenerator;

    /// Fábrica de loggers.
    fn logger_factory(&self) -> impl LoggerFactory;
}

/// Inicializa a `infra` e devolve o seu provider.
///
/// Cria os recursos caros uma vez: o pool e os três caches. Falhar aqui derruba
/// o boot de propósito — melhor não subir do que subir com um banco inalcançável
/// e descobrir isso na primeira requisição.
pub async fn register(secrets: InfraSecrets) -> anyhow::Result<impl InfraProvider> {
    let pool = database::pool::connect(&secrets).await?;

    Ok(InfraProviderImpl {
        pool,
        // Dois registros e dois mapas. Compartilhar um só faria o grupo de
        // marcador `refresh-token` aparecer na listagem de permissões — e, pior,
        // ser concedido ao papel que o `POST /setup` cria, que recebe tudo que
        // estiver registrado. São vocabulários diferentes; o schema anterior os
        // separava em duas tabelas pelo mesmo motivo.
        permission_cache: PermissionCache::new(METADATA_CACHE_CAPACITY),
        marker_group_cache: MarkerGroupCache::new(METADATA_CACHE_CAPACITY),
        marker_cache: build_marker_cache(MARKER_CACHE_CAPACITY),
        read_cache: Arc::new(
            Cache::builder()
                .max_capacity(READ_CACHE_CAPACITY)
                .time_to_live(Duration::from_secs(READ_CACHE_TTL_SECONDS))
                // A invalidação por prefixo depende disto: sem os closures, uma
                // escrita não teria como derrubar todas as listagens afetadas.
                .support_invalidation_closures()
                .build(),
        ),
    })
}

/// A implementação do provider. Privada: nenhum crate exporta impl.
struct InfraProviderImpl {
    pool: MySqlPool,
    permission_cache: PermissionCache,
    marker_group_cache: MarkerGroupCache,
    marker_cache: MarkerCache,
    read_cache: ReadCacheStore,
}

impl InfraProvider for InfraProviderImpl {
    fn unit_of_work(&self) -> impl database::UnitOfWork {
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

    fn query_repository(&self) -> impl query::QueryRepository {
        MariadbQueryRepository::new()
    }

    fn query_factory(&self) -> impl query::QueryFactory {
        MariadbQueryFactory::new()
    }

    fn read_cache(&self) -> impl cache::ReadCache {
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
