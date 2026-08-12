//! A impl do provider da apresentação. Não sai do crate.

use portmaster_app::{AppProvider, Logger, LoggerFactory, SequentialIdGenerator};
use std::time::Duration;

use crate::bootstrap::provider::ApiProvider;
use crate::config::api_config::ApiConfig;
use crate::config::jwt_config::JwtConfig;
use crate::controllers::account_controller::AccountController;
use crate::controllers::auth_controller::AuthController;
use crate::controllers::container_controller::ContainerController;
use crate::controllers::intern::account_controller_impl::AccountControllerImpl;
use crate::controllers::intern::auth_controller_impl::AuthControllerImpl;
use crate::controllers::intern::container_controller_impl::ContainerControllerImpl;
use crate::controllers::intern::manifest_controller_impl::ManifestControllerImpl;
use crate::controllers::intern::metadata_controller_impl::MetadataControllerImpl;
use crate::controllers::intern::metrics_controller_impl::MetricsControllerImpl;
use crate::controllers::intern::product_controller_impl::ProductControllerImpl;
use crate::controllers::intern::role_controller_impl::RoleControllerImpl;
use crate::controllers::intern::server_controller_impl::ServerControllerImpl;
use crate::controllers::intern::user_controller_impl::UserControllerImpl;
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
use crate::ports::token::adapter::jwt_token_service::JwtTokenService;
use crate::ports::token::token_service::TokenService;

/// O nome do logger que os controllers recebem quando precisam de um.
const AUTH_CHANNEL: &str = "auth";

/// O provider da apresentação.
///
/// Três campos, e é a regra: **o provider da camada de baixo e a configuração,
/// nada mais**. Ele não recebe classe pronta — monta as que precisa, aqui
/// dentro, a partir do que a configuração diz.
///
/// Recebia seis argumentos, entre eles um `JwtTokenService` e um
/// `HttpAuthCookie` já construídos. Quem os construía era o `register`, que
/// existia por causa disso: ele destrinchava a configuração em valores soltos —
/// o segredo do token, os nomes de cookie, o ambiente, o teto de tempo, as
/// origens de CORS — e os repassava um a um. Uma configuração nova significava
/// um argumento novo em duas assinaturas, e a construção de um objeto ficava
/// longe de quem sabe do que ele é feito.
///
/// A configuração continua morrendo no boot, só que junto com o provider: os
/// dois são consumidos ao montar o router e nada os mantém vivos depois.
pub(crate) struct ApiProviderImpl<P> {
    /// De onde saem os services.
    app: P,
    /// Onde o servidor escuta e como se comporta.
    config: ApiConfig,
    /// O segredo e o emissor do token de sessão.
    jwt: JwtConfig,
}

impl<P: AppProvider> ApiProviderImpl<P> {
    /// Guarda o provider de baixo e a configuração desta camada.
    pub(crate) const fn new(app: P, config: ApiConfig, jwt: JwtConfig) -> Self {
        Self { app, config, jwt }
    }
}

impl<P: AppProvider> ApiProvider for ApiProviderImpl<P> {
    fn account_controller(&self) -> impl AccountController + use<P> + 'static {
        AccountControllerImpl::new(self.app.account_service(), SessionContext)
    }

    fn auth_controller(&self) -> impl AuthController + use<P> + 'static {
        AuthControllerImpl::new(
            self.app.session_service(),
            self.app.mark_service(),
            self.app.random_id_generator(),
            self.token_service(),
            CookieContext,
            self.logger(AUTH_CHANNEL),
            SessionPolicy::REFRESH_TTL.as_secs(),
        )
    }

    fn container_controller(&self) -> impl ContainerController + use<P> + 'static {
        ContainerControllerImpl::new(self.app.container_service(), SessionContext)
    }

    fn manifest_controller(&self) -> impl ManifestController + use<P> + 'static {
        ManifestControllerImpl::new(self.app.manifest_service(), SessionContext)
    }

    fn metadata_controller(&self) -> impl MetadataController + use<P> + 'static {
        MetadataControllerImpl::new(self.app.metadata_service(), SessionContext)
    }

    fn metrics_controller(&self) -> impl MetricsController + use<P> + 'static {
        MetricsControllerImpl::new(self.app.metrics_service(), SessionContext)
    }

    fn product_controller(&self) -> impl ProductController + use<P> + 'static {
        ProductControllerImpl::new(self.app.product_service(), SessionContext)
    }

    fn role_controller(&self) -> impl RoleController + use<P> + 'static {
        RoleControllerImpl::new(self.app.role_service(), SessionContext)
    }

    fn server_controller(&self) -> impl ServerController + use<P> + 'static {
        ServerControllerImpl::new(self.config.environment.clone())
    }

    fn user_controller(&self) -> impl UserController + use<P> + 'static {
        UserControllerImpl::new(self.app.user_service(), SessionContext)
    }

    /// Construído a cada chamada, e não guardado.
    ///
    /// É barato — duas chaves HMAC a partir do mesmo segredo — e é chamado duas
    /// vezes no boot: uma pelo controller de sessão, outra pelo middleware. Um
    /// campo memoizado economizaria uma construção e custaria um `JwtConfig`
    /// destrinchado em dois lugares.
    fn token_service(&self) -> impl TokenService + use<P> + 'static {
        JwtTokenService::new(&self.jwt)
    }

    fn logger_factory(&self) -> impl LoggerFactory + use<P> + 'static {
        self.app.logger_factory()
    }

    fn logger(&self, name: &str) -> impl Logger + use<P> + 'static {
        self.app.logger_factory().create(name)
    }

    fn sequential_id_generator(&self) -> impl SequentialIdGenerator + use<P> + 'static {
        self.app.sequential_id_generator()
    }

    fn request_timeout(&self) -> Duration {
        self.config.request_timeout
    }

    fn cors_origins(&self) -> &[String] {
        &self.config.cors_origins
    }
}
