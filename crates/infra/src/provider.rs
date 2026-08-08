//! Os factories da `infra`.

use crate::cache::ReadCache;
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
    fn unit_of_work(&self) -> impl UnitOfWork + Sync;

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
    /// Roda qualquer consulta que a [`QueryFactory`] souber descrever, e
    /// nada além delas.
    fn query_repository(&self) -> impl QueryRepository + Sync;

    /// Os descritores de consulta que o `app` pode pedir.
    ///
    /// Separada do repositório porque as duas responsabilidades são distintas:
    /// esta declara **o que** dá para consultar, aquela sabe **executar**. Fundir
    /// as duas num objeto só faria cada consulta nova mexer no executor.
    fn query_factory(&self) -> impl QueryFactory + Send + Sync;

    /// Cache de leitura.
    fn read_cache(&self) -> impl ReadCache + Sync;

    /// Gerador de id opaco, para o refresh token.
    fn random_id_generator(&self) -> impl RandomIdGenerator;

    /// Gerador de id ordenável, para o `request_id`.
    fn sortable_id_generator(&self) -> impl SortableIdGenerator;

    /// Fábrica de loggers.
    fn logger_factory(&self) -> impl LoggerFactory;
}
