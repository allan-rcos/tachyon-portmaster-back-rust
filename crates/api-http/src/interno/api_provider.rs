//! A impl do provider da apresentação. Não sai do crate.

use portmaster_app::{AppProvider, Clock, Logger, LoggerFactory, SortableIdGenerator};
use std::time::Duration;

use crate::controllers::account_controller::AccountController;
use crate::controllers::auth_controller::AuthController;
use crate::controllers::container_controller::ContainerController;
use crate::controllers::interno::account_controller_impl::AccountControllerImpl;
use crate::controllers::interno::auth_controller_impl::AuthControllerImpl;
use crate::controllers::interno::container_controller_impl::ContainerControllerImpl;
use crate::controllers::interno::manifest_controller_impl::ManifestControllerImpl;
use crate::controllers::interno::metadata_controller_impl::MetadataControllerImpl;
use crate::controllers::interno::metrics_controller_impl::MetricsControllerImpl;
use crate::controllers::interno::product_controller_impl::ProductControllerImpl;
use crate::controllers::interno::role_controller_impl::RoleControllerImpl;
use crate::controllers::interno::server_controller_impl::ServerControllerImpl;
use crate::controllers::interno::user_controller_impl::UserControllerImpl;
use crate::controllers::manifest_controller::ManifestController;
use crate::controllers::metadata_controller::MetadataController;
use crate::controllers::metrics_controller::MetricsController;
use crate::controllers::product_controller::ProductController;
use crate::controllers::role_controller::RoleController;
use crate::controllers::server_controller::ServerController;
use crate::controllers::user_controller::UserController;
use crate::cookie::auth_cookie::AuthCookie;
use crate::cookie::interno::http_auth_cookie::HttpAuthCookie;
use crate::provider::ApiProvider;
use crate::token::interno::jwt_token_service::JwtTokenService;
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
/// A [`ApiConfig`] **não** está entre os campos. Ela entra no
/// [`crate::register::register`], é destrinchada nos valores que cada
/// construtor precisa, e morre ali.
pub(crate) struct ApiProviderImpl<P, K> {
    /// De onde saem os casos de uso.
    app: P,
    /// Quem emite e confere o access token.
    ///
    /// Genérico sobre o relógio porque o tipo que o `AppProvider` devolve é
    /// innomeável — só existe depois da monomorfização. É a mesma razão de o
    /// router ser genérico sobre o provider, um andar acima.
    tokens: JwtTokenService<K>,
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

impl<P: AppProvider, K: Clock> ApiProviderImpl<P, K> {
    /// Monta o provider com o que o boot já resolveu.
    ///
    /// O serviço de token chega pronto porque só quem tem a [`ApiConfig`] em
    /// mãos consegue construí-lo — e é o [`crate::register::register`] que a
    /// tem, e que a descarta em seguida.
    pub(crate) const fn new(
        app: P,
        tokens: JwtTokenService<K>,
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

impl<P: AppProvider, K: Clock> ApiProvider for ApiProviderImpl<P, K> {
    fn account_controller(&self) -> impl AccountController + use<P, K> + 'static {
        AccountControllerImpl::new(self.app.account_use_case())
    }

    fn auth_controller(&self) -> impl AuthController + use<P, K> + 'static {
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

    fn container_controller(&self) -> impl ContainerController + use<P, K> + 'static {
        ContainerControllerImpl::new(self.app.container_use_case())
    }

    fn manifest_controller(&self) -> impl ManifestController + use<P, K> + 'static {
        ManifestControllerImpl::new(self.app.manifest_use_case())
    }

    fn metadata_controller(&self) -> impl MetadataController + use<P, K> + 'static {
        MetadataControllerImpl::new(self.app.metadata_use_case())
    }

    fn metrics_controller(&self) -> impl MetricsController + use<P, K> + 'static {
        MetricsControllerImpl::new(self.app.metrics_use_case())
    }

    fn product_controller(&self) -> impl ProductController + use<P, K> + 'static {
        ProductControllerImpl::new(self.app.product_use_case())
    }

    fn role_controller(&self) -> impl RoleController + use<P, K> + 'static {
        RoleControllerImpl::new(self.app.role_use_case())
    }

    fn server_controller(&self) -> impl ServerController + use<P, K> + 'static {
        ServerControllerImpl::new(self.environment.clone())
    }

    fn user_controller(&self) -> impl UserController + use<P, K> + 'static {
        UserControllerImpl::new(self.app.user_use_case())
    }

    fn token_service(&self) -> impl TokenService + use<P, K> + 'static {
        self.tokens.clone()
    }

    fn auth_cookie(&self) -> impl AuthCookie + use<P, K> + 'static {
        self.cookies.clone()
    }

    fn logger_factory(&self) -> impl LoggerFactory + use<P, K> + 'static {
        self.app.logger_factory()
    }

    fn logger(&self, name: &str) -> impl Logger + use<P, K> + 'static {
        self.app.logger_factory().create(name)
    }

    fn sortable_id_generator(&self) -> impl SortableIdGenerator + use<P, K> + 'static {
        self.app.sortable_id_generator()
    }

    fn clock(&self) -> impl Clock + use<P, K> + 'static {
        self.app.clock()
    }

    fn request_timeout(&self) -> Duration {
        self.request_timeout
    }

    fn cors_origins(&self) -> &[String] {
        &self.cors_origins
    }
}
