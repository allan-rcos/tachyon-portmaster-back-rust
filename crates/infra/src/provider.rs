//! Os factories da `infra`.

use crate::cache::ReadCache;
use crate::clock::Clock;
use crate::database::UnitOfWork;
use crate::id::{RandomIdGenerator, SortableIdGenerator};
use crate::logging::LoggerFactory;
use crate::query::{QueryFactory, QueryRepository};
use crate::repository::{
    ContainerRepository, ManifestRepository, MarkerGroupRepository, MarkerRepository,
    PermissionRepository, ProductRepository, RoleRepository, UserRepository,
};

/// Os factories da `infra`.
///
/// Cada método devolve `impl Trait` — contrato, nunca tipo concreto. Os
/// **recursos base** (pool, caches) nascem uma vez no [`crate::register::register`] e são
/// entregues por clone barato; os **serviços** são reconstruídos a cada chamada,
/// porque são structs de campos leves e custam praticamente nada.
pub trait InfraProvider {
    /// A unidade de trabalho da requisição.
    fn unit_of_work(&self) -> impl UnitOfWork + Sync + Clone + use<Self> + 'static;

    /// Persistência de usuários.
    fn user_repository(&self) -> impl UserRepository + Sync + Clone + use<Self> + 'static;

    /// Persistência de papéis.
    fn role_repository(&self) -> impl RoleRepository + Sync + Clone + use<Self> + 'static;

    /// Persistência de produtos.
    fn product_repository(&self) -> impl ProductRepository + Sync + Clone + use<Self> + 'static;

    /// Persistência de contêineres.
    fn container_repository(&self) -> impl ContainerRepository + Sync + Clone + use<Self> + 'static;

    /// Persistência de carga e telemetria.
    fn manifest_repository(&self) -> impl ManifestRepository + Sync + Clone + use<Self> + 'static;

    /// Registro de permissões.
    fn permission_repository(&self) -> impl PermissionRepository + Sync + Clone + use<Self> + 'static;

    /// Registro de grupos de marcador.
    fn marker_group_repository(&self) -> impl MarkerGroupRepository + Sync + Clone + use<Self> + 'static;

    /// Marcadores booleanos com prazo.
    fn marker_repository(&self) -> impl MarkerRepository + Sync + Clone + use<Self> + 'static;

    /// A fonte única do lado de leitura.
    ///
    /// Roda qualquer consulta que a [`QueryFactory`] souber descrever, e
    /// nada além delas.
    fn query_repository(&self) -> impl QueryRepository + Sync + Clone + use<Self> + 'static;

    /// Os descritores de consulta que o `app` pode pedir.
    ///
    /// Separada do repositório porque as duas responsabilidades são distintas:
    /// esta declara **o que** dá para consultar, aquela sabe **executar**. Fundir
    /// as duas num objeto só faria cada consulta nova mexer no executor.
    fn query_factory(&self) -> impl QueryFactory + Send + Sync + Clone + use<Self> + 'static;

    /// Cache de leitura.
    fn read_cache(&self) -> impl ReadCache + Sync + Clone + use<Self> + 'static;

    /// Gerador de id opaco, para o refresh token.
    fn random_id_generator(&self) -> impl RandomIdGenerator + use<Self> + 'static;

    /// Gerador de id ordenável, para o `request_id`.
    fn sortable_id_generator(&self) -> impl SortableIdGenerator + use<Self> + 'static;

    /// Fábrica de loggers.
    fn logger_factory(&self) -> impl LoggerFactory + use<Self> + 'static;

    /// A hora corrente, em UTC.
    fn clock(&self) -> impl Clock + use<Self> + 'static;
}
