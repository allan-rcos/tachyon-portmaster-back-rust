//! Quem serve os controllers.

use std::sync::{PoisonError, RwLock};

use portmaster_app::{AppProvider, LoggerFactory as _};

use crate::config::api_config::ApiConfig;
use crate::controllers::account_controller::AccountController;
use crate::controllers::auth_controller::AuthController;
use crate::controllers::container_controller::ContainerController;
use crate::controllers::intern::account_controller_impl::account_controller;
use crate::controllers::intern::auth_controller_impl::auth_controller;
use crate::controllers::intern::container_controller_impl::container_controller;
use crate::controllers::intern::manifest_controller_impl::manifest_controller;
use crate::controllers::intern::metadata_controller_impl::metadata_controller;
use crate::controllers::intern::metrics_controller_impl::metrics_controller;
use crate::controllers::intern::product_controller_impl::product_controller;
use crate::controllers::intern::role_controller_impl::role_controller;
use crate::controllers::intern::server_controller_impl::server_controller;
use crate::controllers::intern::user_controller_impl::user_controller;
use crate::controllers::manifest_controller::ManifestController;
use crate::controllers::metadata_controller::MetadataController;
use crate::controllers::metrics_controller::MetricsController;
use crate::controllers::product_controller::ProductController;
use crate::controllers::role_controller::RoleController;
use crate::controllers::server_controller::ServerController;
use crate::controllers::user_controller::UserController;
use crate::middleware::intern::cookie_context::CookieContext;
use crate::middleware::intern::session_context::SessionContext;
use crate::ports::session_policy::SessionPolicy;
use crate::ports::token::TokenProvider;

/// O nome do logger que o controller de sessão recebe.
const AUTH_CHANNEL: &str = "auth";

/// O nome do ambiente, para o `/info`.
///
/// É o único dado de [`ApiConfig`] que um controller carrega, e ele precisa
/// estar aqui e não por argumento: quem chama [`ControllersProvider::server`] é
/// a tabela de rotas, que não tem — nem deveria ter — a configuração em mãos.
///
/// `RwLock` porque é configuração, e configuração se troca: instalar de novo
/// muda o que o `/info` responde da próxima requisição em diante.
static ENVIRONMENT: RwLock<Option<String>> = RwLock::new(None);

/// Os controllers, já costurados com os services que consomem.
///
/// Construídos uma vez no boot e **clonados por requisição**, ao contrário dos
/// services, que são reconstruídos a cada chamada. É a diferença que justifica
/// o `self` por valor nos handlers: um controller é um punhado de handles, e
/// clonar é o que o axum faz com todo handler.
pub(crate) struct ControllersProvider;

impl ControllersProvider {
    /// Instala o nome do ambiente que o `/info` responde.
    ///
    /// É a única coisa que um controller carrega da configuração desta camada,
    /// e é por isso que ele entra sozinho: trocar o ambiente não tem por que
    /// obrigar a passar o segredo do token junto. Quem instala o segredo é o
    /// [`TokenProvider`], que é quem o consome.
    pub(crate) fn install_environment(environment: String) {
        *ENVIRONMENT.write().unwrap_or_else(PoisonError::into_inner) = Some(environment);
    }

    /// A conta do próprio usuário.
    pub(crate) fn account() -> anyhow::Result<impl AccountController + use<> + 'static> {
        Ok(account_controller(
            AppProvider::account_service()?,
            SessionContext,
        ))
    }

    /// Sessão: login, setup, refresh e logout.
    pub(crate) fn auth() -> anyhow::Result<impl AuthController + use<> + 'static> {
        Ok(auth_controller(
            AppProvider::session_service()?,
            AppProvider::mark_service(),
            AppProvider::random_id_generator(),
            TokenProvider::token_service()?,
            CookieContext,
            AppProvider::logger_factory().create(AUTH_CHANNEL),
            SessionPolicy::REFRESH_TTL.as_secs(),
        ))
    }

    /// Contêineres.
    pub(crate) fn container() -> anyhow::Result<impl ContainerController + use<> + 'static> {
        Ok(container_controller(
            AppProvider::container_service()?,
            SessionContext,
        ))
    }

    /// Carga e telemetria.
    pub(crate) fn manifest() -> anyhow::Result<impl ManifestController + use<> + 'static> {
        Ok(manifest_controller(
            AppProvider::manifest_service()?,
            SessionContext,
        ))
    }

    /// Metadados de sistema.
    pub(crate) fn metadata() -> impl MetadataController + use<> + 'static {
        metadata_controller(AppProvider::metadata_service(), SessionContext)
    }

    /// O painel do pátio.
    pub(crate) fn metrics() -> anyhow::Result<impl MetricsController + use<> + 'static> {
        Ok(metrics_controller(
            AppProvider::metrics_service()?,
            SessionContext,
        ))
    }

    /// Produtos.
    pub(crate) fn product() -> anyhow::Result<impl ProductController + use<> + 'static> {
        Ok(product_controller(
            AppProvider::product_service()?,
            SessionContext,
        ))
    }

    /// Papéis.
    pub(crate) fn role() -> anyhow::Result<impl RoleController + use<> + 'static> {
        Ok(role_controller(
            AppProvider::role_service()?,
            SessionContext,
        ))
    }

    /// O estado do próprio processo.
    ///
    /// Sem o nome do ambiente instalado, vale o padrão de [`ApiConfig`] — o
    /// mesmo que valeria se a variável não estivesse no ambiente. É só o que o
    /// `/info` mostra, e um `/info` que não responde seria pior do que um que
    /// responde o padrão.
    pub(crate) fn server() -> impl ServerController + use<> + 'static {
        server_controller(
            ENVIRONMENT
                .read()
                .unwrap_or_else(PoisonError::into_inner)
                .clone()
                .unwrap_or_else(|| ApiConfig::default().environment),
        )
    }

    /// Usuários.
    pub(crate) fn user() -> anyhow::Result<impl UserController + use<> + 'static> {
        Ok(user_controller(
            AppProvider::user_service()?,
            SessionContext,
        ))
    }
}
