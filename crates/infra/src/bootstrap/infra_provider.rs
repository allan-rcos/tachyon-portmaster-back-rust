//! A borda da `infra`: tudo que o `app` alcança sai por aqui.

use crate::config::InfraSecrets;
use crate::logging::LoggerFactory;
use crate::logging::LoggingProvider;
use crate::query::QueryProvider;
use crate::query::QueryRepository;
use crate::repository::RepositoryProvider;
use crate::repository::{
    ContainerRepository, ManifestRepository, MarkerGroupRepository, MarkerRepository,
    PermissionRepository, ProductRepository, RoleRepository, UserRepository, ViewCacheRepository,
};
use crate::scope::ScopeProvider;

/// Os factories da `infra`.
///
/// O `app` recebe **contrato**: nenhum tipo concreto desta camada atravessa a
/// fronteira. Ver `docs/adr/0011-static-providers-one-per-directory.md`.
///
/// Metade dos factories devolve `Result` e a outra metade não. A divisão não é
/// descuido: quem depende do pool pode falhar antes de o boot ter instalado os
/// segredos, e quem vive em memória não tem como. Uma assinatura uniforme
/// pediria `?` a chamadas que não podem dar errado.
///
/// Não há `unit_of_work` aqui, e continua não havendo: quem entrega a unidade
/// de trabalho é o `MasterScope::run`, a partir das layers que o linker
/// recolhe. O que os repositórios carregam é o handle que alcança a transação
/// da tarefa.
pub struct InfraProvider;

impl InfraProvider {
    /// Abre o pool com estes segredos, e larga o que estava aberto.
    ///
    /// É a única configuração da camada, e por isso o único `install` daqui —
    /// os quatro mapas em memória não têm o que configurar. Instalar de novo
    /// troca, e é o que permite apontar o processo para outro banco sem
    /// reiniciá-lo.
    ///
    /// Abrir é preguiçoso: não toca a rede. Quem confirma que há um banco do
    /// outro lado é o [`Self::check_database`].
    pub fn install_database(secrets: &InfraSecrets) -> anyhow::Result<()> {
        ScopeProvider::install_database(secrets)
    }

    /// Confirma que o banco responde, e derruba o boot se não responder.
    ///
    /// Falhar aqui é de propósito: subir com um banco inalcançável só adia a
    /// descoberta para a primeira requisição, com o processo já reportado como
    /// saudável.
    pub async fn check_database() -> anyhow::Result<()> {
        ScopeProvider::ping().await
    }

    /// A persistência de usuários.
    pub fn user_repository() -> anyhow::Result<impl UserRepository + Sync + Clone + use<> + 'static>
    {
        RepositoryProvider::user()
    }

    /// A persistência de papéis.
    pub fn role_repository() -> anyhow::Result<impl RoleRepository + Sync + Clone + use<> + 'static>
    {
        RepositoryProvider::role()
    }

    /// A persistência de produtos.
    pub fn product_repository(
    ) -> anyhow::Result<impl ProductRepository + Sync + Clone + use<> + 'static> {
        RepositoryProvider::product()
    }

    /// A persistência de contêineres.
    pub fn container_repository(
    ) -> anyhow::Result<impl ContainerRepository + Sync + Clone + use<> + 'static> {
        RepositoryProvider::container()
    }

    /// A persistência de manifesto.
    pub fn manifest_repository(
    ) -> anyhow::Result<impl ManifestRepository + Sync + Clone + use<> + 'static> {
        RepositoryProvider::manifest()
    }

    /// O catálogo de permissões.
    pub fn permission_repository() -> impl PermissionRepository + Sync + Clone + use<> + 'static {
        RepositoryProvider::permission()
    }

    /// O registro de grupos de marcador.
    pub fn marker_group_repository() -> impl MarkerGroupRepository + Sync + Clone + use<> + 'static
    {
        RepositoryProvider::marker_group()
    }

    /// Os marcadores.
    pub fn marker_repository() -> impl MarkerRepository + Sync + Clone + use<> + 'static {
        RepositoryProvider::marker()
    }

    /// Quem executa um DQL contra o banco.
    pub fn query_repository(
    ) -> anyhow::Result<impl QueryRepository + Sync + Clone + use<> + 'static> {
        QueryProvider::queries()
    }

    /// O cache do lado de leitura.
    pub fn view_cache_repository() -> impl ViewCacheRepository + Sync + Clone + use<> + 'static {
        RepositoryProvider::view_cache()
    }

    /// A fábrica de loggers nomeados.
    pub fn logger_factory() -> impl LoggerFactory + use<> {
        LoggingProvider::logger_factory()
    }
}
