//! A borda do `app`: tudo que a apresentação alcança sai por aqui, e o boot do
//! processo começa aqui.

use anyhow::Context as _;
use portmaster_domain::id::{RandomIdGenerator, SequentialIdGenerator};
use portmaster_domain::DomainProvider;
use portmaster_infra::logging::LoggerFactory;
use portmaster_infra::InfraProvider;

use crate::config::AppSecrets;
use crate::event::{EventProvider, MetaEventStackSubscriber};
use crate::services::ServicesProvider;
use crate::services::{
    AccountService, ContainerService, ManifestService, MarkService, MetadataService,
    MetricsService, ProductService, RoleService, SessionService, UserService,
};

/// Os factories dos services.
///
/// A apresentação recebe **contrato**: nenhum tipo concreto de service
/// atravessa esta fronteira. Ver
/// `docs/adr/0011-static-providers-one-per-directory.md`.
///
/// Oito dos dez devolvem `Result`, e dois não. A divisão vem de baixo: quem
/// depende do pool pode falhar se o boot ainda não tiver instalado os segredos
/// do banco, e quem só toca memória não tem como. Todas as chamadas acontecem
/// no boot, dentro de funções que já devolvem `anyhow::Result`.
pub struct AppProvider;

impl AppProvider {
    /// Inicializa o sistema inteiro.
    ///
    /// Instala a configuração das camadas de baixo em quem a consome, confirma
    /// que o banco responde e declara o catálogo. O `main` chama isto e depois
    /// monta o router, sem conhecer `domain` nem `infra`.
    ///
    /// ## O catálogo de permissões nasce aqui, e esta função não conhece um slug
    ///
    /// Cada serviço declara as suas, e o que este boot faz é pedir que declarem.
    /// É o molde do `declarePermission` do PHP: a permissão pertence a exatamente
    /// um caso de uso, e é ele quem a nomeia. Acrescentar um serviço é acrescentar
    /// uma linha aqui — não editar uma lista central que ninguém lembra de manter.
    ///
    /// O catálogo precisa existir antes da primeira requisição: o `POST /setup`
    /// concede ao primeiro papel tudo que estiver registrado. Falhar aqui derruba o
    /// boot de propósito — um sistema que subiu com o catálogo pela metade daria 403
    /// em endpoints que deveriam funcionar, e a causa seria invisível.
    ///
    /// Registrar no boot, e não no construtor de cada caso de uso como o PHP fazia,
    /// evita repetir o registro a cada requisição: os services são reconstruídos
    /// o tempo todo, o catálogo não.
    ///
    /// **Grupo de marcador não entra aqui.** Quem usa um grupo é a apresentação, e é
    /// ela quem o declara — esta camada não sabe o que é sessão.
    ///
    /// É o único ponto em que um erro tipado vira `anyhow`: aqui é wiring de root, e
    /// o que sai daqui vai para o `main`.
    pub async fn boot(secrets: &AppSecrets) -> anyhow::Result<()> {
        DomainProvider::install_identity(secrets.domain);
        InfraProvider::install_database(&secrets.infra)?;
        InfraProvider::check_database().await?;

        let metadata = Self::metadata_service();

        metadata
            .declare_permissions()
            .await
            .context("falha ao registrar as permissões de metadados")?;

        Self::container_service()?
            .declare_permissions(&metadata)
            .await
            .context("falha ao registrar as permissões de contêiner")?;

        Self::manifest_service()?
            .declare_permissions(&metadata)
            .await
            .context("falha ao registrar as permissões de manifesto")?;

        Self::metrics_service()?
            .declare_permissions(&metadata)
            .await
            .context("falha ao registrar as permissões do painel")?;

        Self::product_service()?
            .declare_permissions(&metadata)
            .await
            .context("falha ao registrar as permissões de produto")?;

        Self::role_service()?
            .declare_permissions(&metadata)
            .await
            .context("falha ao registrar as permissões de papel")?;

        Self::user_service()?
            .declare_permissions(&metadata)
            .await
            .context("falha ao registrar as permissões de usuário")?;

        Ok(())
    }

    /// A conta do próprio usuário.
    pub fn account_service() -> anyhow::Result<impl AccountService + Sync + Clone + use<> + 'static>
    {
        ServicesProvider::account()
    }

    /// Contêineres.
    pub fn container_service(
    ) -> anyhow::Result<impl ContainerService + Sync + Clone + use<> + 'static> {
        ServicesProvider::container()
    }

    /// Carga e telemetria.
    pub fn manifest_service(
    ) -> anyhow::Result<impl ManifestService + Sync + Clone + use<> + 'static> {
        ServicesProvider::manifest()
    }

    /// A primitiva de marcação — sessão de refresh é um uso dela.
    pub fn mark_service() -> impl MarkService + Sync + Clone + use<> + 'static {
        ServicesProvider::mark()
    }

    /// Metadados de sistema.
    pub fn metadata_service() -> impl MetadataService + Sync + Clone + use<> + 'static {
        ServicesProvider::metadata()
    }

    /// O painel do pátio.
    pub fn metrics_service() -> anyhow::Result<impl MetricsService + Sync + Clone + use<> + 'static>
    {
        ServicesProvider::metrics()
    }

    /// Produtos.
    pub fn product_service() -> anyhow::Result<impl ProductService + Sync + Clone + use<> + 'static>
    {
        ServicesProvider::product()
    }

    /// Papéis.
    pub fn role_service() -> anyhow::Result<impl RoleService + Sync + Clone + use<> + 'static> {
        ServicesProvider::role()
    }

    /// Login, validação de sessão e o setup inicial.
    pub fn session_service() -> anyhow::Result<impl SessionService + Sync + Clone + use<> + 'static>
    {
        ServicesProvider::session()
    }

    /// Usuários.
    pub fn user_service() -> anyhow::Result<impl UserService + Sync + Clone + use<> + 'static> {
        ServicesProvider::user()
    }

    /// Fábrica de loggers, para a apresentação.
    pub fn logger_factory() -> impl LoggerFactory + use<> {
        InfraProvider::logger_factory()
    }

    /// Gerador de id opaco, para o refresh token.
    pub fn random_id_generator() -> impl RandomIdGenerator + use<> {
        DomainProvider::random_id_generator()
    }

    /// Gerador de id ordenável, para o `request_id`.
    pub fn sequential_id_generator() -> impl SequentialIdGenerator + use<> {
        DomainProvider::sequential_id_generator()
    }

    /// A pilha de eventos da tarefa, só o lado de leitura.
    ///
    /// O que atravessa é `MetaEventStackSubscriber` e mais nada: abrir o escopo
    /// e perguntar o que foi registrado são assunto da apresentação, emitir não
    /// é. O trait de escrita nem sai do crate, então esta assinatura é a única
    /// forma de a pilha existir do lado de fora.
    pub fn meta_event_stack() -> impl MetaEventStackSubscriber + Sync + Clone + use<> + 'static {
        EventProvider::meta_event_stack()
    }
}
