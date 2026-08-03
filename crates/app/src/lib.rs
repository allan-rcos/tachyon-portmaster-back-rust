//! # portmaster-app
//!
//! Orquestração. Depende de `domain` (TableModules, traits de objeto) e de
//! `infra` (repositories, `UnitOfWork`, cache).
//!
//! Dono dos **Command DTOs**, dos **Query DTOs** e dos traits de UseCase — que
//! são exatamente os ports de que a apresentação precisa, e por isso são
//! definidos aqui: o `api-http` conhece só esta camada.
//!
//! É aqui que a autorização acontece. Cada UseCase protegido declara no
//! construtor a permissão que exige e a confere na primeira linha, contra o
//! `UserContext` que veio no Command — o contexto chega por argumento, nunca de
//! um estado global. É também o único lugar que abre e fecha transação.
//!
//! ## O que esta camada reexporta, e por quê
//!
//! O `api-http` conhece só o `app`. Então tudo que ele precisa e que nasce
//! abaixo — o trait `User` para mapear ao fio, as Views de leitura, o `Logger`,
//! os geradores de id — sai daqui. Não é conveniência: é o que mantém o grafo de
//! dependências com uma seta só entre cada par de camadas.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod account;
pub mod authorization;
pub mod container;
pub mod context;
pub mod error;
pub mod manifest;
pub mod marker;
pub mod metadata;
pub mod metrics;
pub mod product;
pub mod role;
pub mod session;
pub mod user;

pub(crate) mod cache;
pub(crate) mod transaction;

use portmaster_domain::metadata::marker_group::MarkerGroupTM;
use portmaster_domain::metadata::permission::PermissionTM;
use portmaster_domain::DomainProvider;
use portmaster_infra::repository::{MarkerGroupRepository, PermissionRepository};
use portmaster_infra::InfraProvider;

use account::{AccountUseCase, AccountUseCaseImpl};
use container::{ContainerUseCase, ContainerUseCaseImpl};
use manifest::{ManifestUseCase, ManifestUseCaseImpl};
use marker::{MarkUseCase, MarkUseCaseImpl};
use metadata::{MetadataUseCase, MetadataUseCaseImpl};
use metrics::{MetricsUseCase, MetricsUseCaseImpl};
use product::{ProductUseCase, ProductUseCaseImpl};
use role::{RoleUseCase, RoleUseCaseImpl};
use session::{SessionUseCase, SessionUseCaseImpl};
use user::{UserUseCase, UserUseCaseImpl};

// --- Reexports para a apresentação -----------------------------------------

/// Os objetos de domínio que um caso de uso devolve.
///
/// Read-only e mapeados imediatamente para o fio — é a exceção de borda que a
/// DI estática admite, e a única razão de haver `Box<dyn>` no sistema.
pub mod domain {
    pub use portmaster_domain::container::Container;
    pub use portmaster_domain::enums::{ContainerStatus, RiskClass, TelemetryEvent};
    pub use portmaster_domain::error::FieldError;
    pub use portmaster_domain::manifest::{ManifestCargo, ManifestChange};
    pub use portmaster_domain::product::Product;
    pub use portmaster_domain::role::Role;
    pub use portmaster_domain::user::User;
}

/// Os read models do lado de leitura.
pub use portmaster_infra::query::views;

/// O log estruturado.
pub use portmaster_infra::logging::{Logger, LoggerFactory};

/// Os geradores de id que não são identidade de entidade.
///
/// Vivem na `infra` e passam por aqui: o refresh token opaco e o `request_id`
/// são da apresentação, não do domínio.
pub use portmaster_infra::id::{RandomIdGenerator, SortableIdGenerator};

/// Os segredos das camadas de baixo.
///
/// Reexportados porque quem os **preenche** é a apresentação, e ela conhece só
/// esta camada. Sem isto, o `main` teria que declarar `portmaster-infra` como
/// dependência para nomear um tipo que só repassa — e a seta a mais no grafo
/// valeria para sempre, por uma struct de configuração.
pub use portmaster_domain::DomainSecrets;
pub use portmaster_infra::config::{DatabaseSslMode, InfraSecrets};

/// O texto secreto que a URI de conexão carrega.
///
/// Vem do `secrecy`: não implementa `Debug` nem `Display` úteis, o que impede a
/// senha do banco de sair num log por acidente.
pub use portmaster_infra::config::SecretString;

// --- Bootstrap --------------------------------------------------------------

/// Os segredos de todas as camadas, montados pela apresentação.
///
/// O `app` os recebe inteiros e distribui: é ele que encadeia os `register` das
/// camadas de baixo, então é ele que precisa saber do que cada uma vive. A
/// apresentação lê variáveis de ambiente e preenche isto — sem conhecer
/// `domain` nem `infra`.
#[derive(Debug, Clone)]
pub struct AppSecrets {
    /// Identidade de deploy, para o gerador de Snowflake.
    pub domain: DomainSecrets,
    /// Conexão com o banco.
    pub infra: InfraSecrets,
}

/// Os factories dos casos de uso.
///
/// Cada método devolve `impl Trait` — contrato, nunca tipo concreto. O tipo real
/// é innomeável: só existe depois da monomorfização, e é exatamente por isso que
/// a apresentação não consegue depender dele.
///
/// Os casos de uso são reconstruídos a cada chamada. Isso é barato de propósito:
/// eles não guardam estado, e os recursos caros — pool, caches — nasceram uma
/// vez no [`register`] e chegam por clone.
pub trait AppProvider {
    /// A conta do próprio usuário.
    fn account_use_case(&self) -> impl AccountUseCase + Sync;

    /// Contêineres.
    fn container_use_case(&self) -> impl ContainerUseCase + Sync;

    /// Carga e telemetria.
    fn manifest_use_case(&self) -> impl ManifestUseCase + Sync;

    /// A primitiva de marcação — sessão de refresh é um uso dela.
    fn mark_use_case(&self) -> impl MarkUseCase + Sync;

    /// Metadados de sistema.
    fn metadata_use_case(&self) -> impl MetadataUseCase + Sync;

    /// O painel do pátio.
    fn metrics_use_case(&self) -> impl MetricsUseCase + Sync;

    /// Produtos.
    fn product_use_case(&self) -> impl ProductUseCase + Sync;

    /// Papéis.
    fn role_use_case(&self) -> impl RoleUseCase + Sync;

    /// Login, validação de sessão e o setup inicial.
    fn session_use_case(&self) -> impl SessionUseCase + Sync;

    /// Usuários.
    fn user_use_case(&self) -> impl UserUseCase + Sync;

    /// Fábrica de loggers, para a apresentação.
    fn logger_factory(&self) -> impl LoggerFactory;

    /// Gerador de id opaco, para o refresh token.
    fn random_id_generator(&self) -> impl RandomIdGenerator;

    /// Gerador de id ordenável, para o `request_id`.
    fn sortable_id_generator(&self) -> impl SortableIdGenerator;
}

/// Inicializa o sistema inteiro e devolve o provider da aplicação.
///
/// Encadeia os `register` das camadas de baixo e **embute** os subproviders.
/// Não há composition root: o `main` chama isto e recebe algo pronto, sem
/// conhecer `domain` nem `infra`.
///
/// ## O que acontece aqui e em nenhum outro lugar
///
/// O **catálogo de permissões** e o **grupo de marcador da sessão** são
/// registrados no boot. Os dois precisam existir antes da primeira requisição:
/// o `POST /setup` concede ao primeiro papel tudo que estiver registrado, e o
/// repositório de marcadores recusa marcar num grupo desconhecido.
///
/// Registrar aqui, e não no construtor de cada caso de uso como o PHP fazia,
/// evita repetir o registro a cada requisição — os casos de uso são
/// reconstruídos o tempo todo, o catálogo não.
pub async fn register(secrets: AppSecrets) -> anyhow::Result<impl AppProvider> {
    let domain = portmaster_domain::register(secrets.domain);
    let infra = portmaster_infra::register(secrets.infra).await?;

    declare_metadata(&domain, &infra).await?;

    Ok(AppProviderImpl { domain, infra })
}

/// Preenche o catálogo de permissões e os grupos de marcador.
///
/// Falhar aqui derruba o boot de propósito: um sistema que subiu sem o catálogo
/// completo daria 403 em endpoints que deveriam funcionar, e a causa seria
/// invisível — o papel do administrador simplesmente não teria a permissão.
async fn declare_metadata<D: DomainProvider, I: InfraProvider>(
    domain: &D,
    infra: &I,
) -> anyhow::Result<()> {
    let permission_tm = domain.permission_table_module();
    let permissions = infra.permission_repository();

    for slug in authorization::CATALOG {
        let permission = permission_tm.create((*slug).to_owned())?;
        permissions.register(permission.as_ref()).await?;
    }

    let group_tm = domain.marker_group_table_module();
    let groups = infra.marker_group_repository();
    let group = group_tm.create(authorization::REFRESH_TOKEN_GROUP.to_owned())?;
    groups.register(group.as_ref()).await?;

    Ok(())
}

/// A implementação do provider. Privada: nenhum crate exporta impl.
struct AppProviderImpl<D, I> {
    domain: D,
    infra: I,
}

impl<D: DomainProvider, I: InfraProvider> AppProvider for AppProviderImpl<D, I> {
    fn account_use_case(&self) -> impl AccountUseCase {
        AccountUseCaseImpl::new(
            self.infra.user_repository(),
            self.domain.user_table_module(),
            self.domain.auth_table_module(),
            self.infra.query_repository(),
            self.infra.query_factory(),
            self.infra.read_cache(),
            self.infra.unit_of_work(),
        )
    }

    fn container_use_case(&self) -> impl ContainerUseCase {
        ContainerUseCaseImpl::new(
            self.infra.container_repository(),
            self.domain.container_table_module(),
            self.infra.query_repository(),
            self.infra.query_factory(),
            self.infra.read_cache(),
            self.infra.unit_of_work(),
        )
    }

    fn manifest_use_case(&self) -> impl ManifestUseCase {
        ManifestUseCaseImpl::new(
            self.infra.container_repository(),
            self.infra.product_repository(),
            self.infra.manifest_repository(),
            self.domain.manifest_table_module(),
            self.infra.read_cache(),
            self.infra.unit_of_work(),
        )
    }

    fn mark_use_case(&self) -> impl MarkUseCase {
        MarkUseCaseImpl::new(
            self.domain.marker_table_module(),
            self.infra.marker_repository(),
        )
    }

    fn metadata_use_case(&self) -> impl MetadataUseCase {
        MetadataUseCaseImpl::new(self.infra.permission_repository())
    }

    fn metrics_use_case(&self) -> impl MetricsUseCase {
        MetricsUseCaseImpl::new(
            self.infra.query_repository(),
            self.infra.query_factory(),
            self.infra.read_cache(),
            self.infra.unit_of_work(),
        )
    }

    fn product_use_case(&self) -> impl ProductUseCase {
        ProductUseCaseImpl::new(
            self.infra.product_repository(),
            self.domain.product_table_module(),
            self.infra.query_repository(),
            self.infra.query_factory(),
            self.infra.read_cache(),
            self.infra.unit_of_work(),
        )
    }

    fn role_use_case(&self) -> impl RoleUseCase {
        RoleUseCaseImpl::new(
            self.infra.role_repository(),
            self.domain.role_table_module(),
            self.infra.query_repository(),
            self.infra.query_factory(),
            self.infra.read_cache(),
            self.infra.unit_of_work(),
        )
    }

    fn session_use_case(&self) -> impl SessionUseCase {
        SessionUseCaseImpl::new(
            self.infra.user_repository(),
            self.infra.role_repository(),
            self.infra.permission_repository(),
            self.domain.user_table_module(),
            self.domain.role_table_module(),
            self.domain.auth_table_module(),
            self.infra.unit_of_work(),
        )
    }

    fn user_use_case(&self) -> impl UserUseCase {
        UserUseCaseImpl::new(
            self.infra.user_repository(),
            self.infra.role_repository(),
            self.domain.user_table_module(),
            self.infra.query_repository(),
            self.infra.query_factory(),
            self.infra.read_cache(),
            self.infra.unit_of_work(),
        )
    }

    fn logger_factory(&self) -> impl LoggerFactory {
        self.infra.logger_factory()
    }

    fn random_id_generator(&self) -> impl RandomIdGenerator {
        self.infra.random_id_generator()
    }

    fn sortable_id_generator(&self) -> impl SortableIdGenerator {
        self.infra.sortable_id_generator()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;
    use std::sync::Arc;

    /// Reproduz o que o `api-http` fará com o provider.
    ///
    /// É o teste que mais importa nesta camada, e ele não roda nada: se compila,
    /// passou. A DI é 100% estática, então `Send` do futuro de um caso de uso é
    /// **inferido** do tipo concreto — e a inferência só acontece porque tudo
    /// aqui é genérico, sem `dyn`. Se um port passasse a segurar algo `!Send`
    /// através de um `.await`, todo handler do axum deixaria de compilar, e o
    /// erro apareceria lá — a três camadas de distância da causa.
    ///
    /// Ver tmp/architecture/lifetimes-auto-traits.md: o bound de `Send` mora no
    /// ponto de uso, que é exatamente isto.
    #[allow(dead_code)]
    async fn o_provider_serve_um_handler_do_axum<P>(provider: Arc<P>) -> Result<(), AppError>
    where
        P: AppProvider + Send + Sync + 'static,
    {
        let context = context::UserContext {
            id: "1".into(),
            name: "Ana".into(),
            email: "ana@portmaster.local".into(),
            roles: Vec::new(),
        };

        // `tokio::spawn` exige `Send + 'static`: é a prova de que o futuro do
        // caso de uso atravessa uma fronteira de execução, como num handler.
        let handle = tokio::spawn(async move {
            provider
                .product_use_case()
                .list(product::ListProductsQuery {
                    context,
                    cursor: None,
                    limit: None,
                    search: None,
                })
                .await
        });

        handle.await.expect("a tarefa não deve entrar em pânico")?;

        Ok(())
    }

    /// O mesmo, para um caso de uso de escrita — que devolve `Box<dyn …>`.
    ///
    /// A exceção de borda do `dyn` não pode custar a `Send`-ness do retorno,
    /// senão o objeto de domínio não atravessaria até a apresentação.
    #[allow(dead_code)]
    async fn a_escrita_tambem_atravessa_uma_tarefa<P>(provider: Arc<P>) -> Result<(), AppError>
    where
        P: AppProvider + Send + Sync + 'static,
    {
        let handle = tokio::spawn(async move {
            provider
                .session_use_case()
                .login(session::LoginCommand {
                    email: "ana@portmaster.local".into(),
                    password: "Portmaster1".into(),
                })
                .await
        });

        let user = handle.await.expect("a tarefa não deve entrar em pânico")?;

        // O trait reexportado é o que a apresentação usa para mapear ao fio.
        let _: &str = user.id();

        Ok(())
    }
}
