//! A impl do provider da apresentação. Não sai do crate.

use portmaster_app::{AppProvider, Logger, LoggerFactory, SequentialIdGenerator};
use std::time::Duration;

use crate::bootstrap::provider::ApiProvider;
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
use crate::cookie::auth_cookie::AuthCookie;
use crate::cookie::intern::http_auth_cookie::HttpAuthCookie;
use crate::token::intern::jwt_token_service::JwtTokenService;
use crate::token::token_service::TokenService;

/// O nome do logger que os controllers recebem quando precisam de um.
const AUTH_CHANNEL: &str = "auth";

/// O provider da apresentação.
///
/// Guarda os **recursos já resolvidos** — o serviço de token, os cookies, o
/// ambiente — e o provider da camada de baixo. É a mesma forma dos providers de
/// `domain`, `infra` e `app`, e pela mesma razão: os factories devolvem
/// `impl Trait`, e o tipo concreto de cada controller nunca é nomeado.
///
/// A [`ApiConfig`](crate::config::api_config::ApiConfig) **não** está entre os campos. Ela entra no
/// [`crate::bootstrap::register::register()`], é destrinchada nos valores que cada
/// construtor precisa, e morre ali.
pub(crate) struct ApiProviderImpl<P> {
    /// De onde saem os casos de uso.
    app: P,
    /// Quem emite e confere o access token.
    tokens: JwtTokenService,
    /// Como os cookies de sessão são escritos e lidos.
    cookies: HttpAuthCookie,
    /// Em que ambiente o processo está rodando.
    environment: String,
    /// Por quanto tempo o refresh vale, em segundos.
    refresh_ttl_seconds: u64,
    /// Por quanto tempo uma requisição pode demorar.
    request_timeout: Duration,
    /// As origens que o CORS libera.
    cors_origins: Vec<String>,
}

impl<P: AppProvider> ApiProviderImpl<P> {
    /// Monta o provider com o que o boot já resolveu.
    ///
    /// O serviço de token chega pronto porque só quem tem a [`ApiConfig`](crate::config::api_config::ApiConfig) em
    /// mãos consegue construí-lo — e é o [`crate::bootstrap::register::register()`] que a
    /// tem, e que a descarta em seguida.
    pub(crate) const fn new(
        app: P,
        tokens: JwtTokenService,
        cookies: HttpAuthCookie,
        environment: String,
        refresh_ttl_seconds: u64,
        request_timeout: Duration,
        cors_origins: Vec<String>,
    ) -> Self {
        Self {
            app,
            tokens,
            cookies,
            environment,
            refresh_ttl_seconds,
            request_timeout,
            cors_origins,
        }
    }
}

impl<P: AppProvider> ApiProvider for ApiProviderImpl<P> {
    fn account_controller(&self) -> impl AccountController + use<P> + 'static {
        AccountControllerImpl::new(self.app.account_use_case())
    }

    fn auth_controller(&self) -> impl AuthController + use<P> + 'static {
        AuthControllerImpl::new(
            self.app.session_use_case(),
            self.app.mark_use_case(),
            self.app.random_id_generator(),
            self.tokens.clone(),
            self.cookies.clone(),
            self.logger(AUTH_CHANNEL),
            self.refresh_ttl_seconds,
        )
    }

    fn container_controller(&self) -> impl ContainerController + use<P> + 'static {
        ContainerControllerImpl::new(self.app.container_use_case())
    }

    fn manifest_controller(&self) -> impl ManifestController + use<P> + 'static {
        ManifestControllerImpl::new(self.app.manifest_use_case())
    }

    fn metadata_controller(&self) -> impl MetadataController + use<P> + 'static {
        MetadataControllerImpl::new(self.app.metadata_use_case())
    }

    fn metrics_controller(&self) -> impl MetricsController + use<P> + 'static {
        MetricsControllerImpl::new(self.app.metrics_use_case())
    }

    fn product_controller(&self) -> impl ProductController + use<P> + 'static {
        ProductControllerImpl::new(self.app.product_use_case())
    }

    fn role_controller(&self) -> impl RoleController + use<P> + 'static {
        RoleControllerImpl::new(self.app.role_use_case())
    }

    fn server_controller(&self) -> impl ServerController + use<P> + 'static {
        ServerControllerImpl::new(self.environment.clone())
    }

    fn user_controller(&self) -> impl UserController + use<P> + 'static {
        UserControllerImpl::new(self.app.user_use_case())
    }

    fn token_service(&self) -> impl TokenService + use<P> + 'static {
        self.tokens.clone()
    }

    fn auth_cookie(&self) -> impl AuthCookie + use<P> + 'static {
        self.cookies.clone()
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
        self.request_timeout
    }

    fn cors_origins(&self) -> &[String] {
        &self.cors_origins
    }
}
