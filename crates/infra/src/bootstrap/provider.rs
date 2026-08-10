//! Os factories da `infra`.

use crate::logging::LoggerFactory;
use crate::query::QueryRepository;
use crate::repository::{
    ContainerRepository, ManifestRepository, MarkerGroupRepository, MarkerRepository,
    PermissionRepository, ProductRepository, RoleRepository, UserRepository, ViewCacheRepository,
};

/// Os factories da `infra`.
///
/// Cada método devolve `impl Trait` — contrato, nunca tipo concreto. Os
/// **recursos base** (pool, caches) nascem uma vez no [`crate::register()`] e são
/// entregues por clone barato; os **serviços** são reconstruídos a cada chamada,
/// porque são structs de campos leves e custam praticamente nada.
///
/// Não há `unit_of_work` aqui: quem a entrega é o
/// [`MasterScope::run`](crate::scope::MasterScope::run), a partir das camadas
/// que o linker registrou. O que a `infra` injeta é o handle que **abre** a
/// transação, e ele é detalhe de cada repositório.
pub trait InfraProvider {
    /// Persistência de usuários.
    fn user_repository(&self) -> impl UserRepository + Sync + Clone + use<Self> + 'static;

    /// Persistência de papéis.
    fn role_repository(&self) -> impl RoleRepository + Sync + Clone + use<Self> + 'static;

    /// Persistência de produtos.
    fn product_repository(&self) -> impl ProductRepository + Sync + Clone + use<Self> + 'static;

    /// Persistência de contêineres.
    fn container_repository(&self)
        -> impl ContainerRepository + Sync + Clone + use<Self> + 'static;

    /// Persistência de carga e telemetria.
    fn manifest_repository(&self) -> impl ManifestRepository + Sync + Clone + use<Self> + 'static;

    /// Registro de permissões.
    fn permission_repository(
        &self,
    ) -> impl PermissionRepository + Sync + Clone + use<Self> + 'static;

    /// Registro de grupos de marcador.
    fn marker_group_repository(
        &self,
    ) -> impl MarkerGroupRepository + Sync + Clone + use<Self> + 'static;

    /// Marcadores booleanos com prazo.
    fn marker_repository(&self) -> impl MarkerRepository + Sync + Clone + use<Self> + 'static;

    /// A fonte única do lado de leitura.
    ///
    /// Roda qualquer consulta que uma função de [`dql`](crate::query::dql)
    /// souber descrever, e nada além delas.
    fn query_repository(&self) -> impl QueryRepository + Sync + Clone + use<Self> + 'static;

    /// O cache do lado de leitura.
    ///
    /// Um só para todas as `View`: o que ele guarda é sempre o que uma consulta
    /// devolveu, e a identidade dela vem do próprio DQL.
    fn view_cache_repository(
        &self,
    ) -> impl ViewCacheRepository + Sync + Clone + use<Self> + 'static;

    /// Fábrica de loggers.
    fn logger_factory(&self) -> impl LoggerFactory + use<Self> + 'static;
}
