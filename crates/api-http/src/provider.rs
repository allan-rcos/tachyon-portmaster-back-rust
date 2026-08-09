//! Os factories dos controllers.

use crate::controllers::account_controller::AccountController;
use crate::controllers::auth_controller::AuthController;
use crate::controllers::container_controller::ContainerController;
use crate::controllers::manifest_controller::ManifestController;
use crate::controllers::metadata_controller::MetadataController;
use crate::controllers::metrics_controller::MetricsController;
use crate::controllers::product_controller::ProductController;
use crate::controllers::role_controller::RoleController;
use crate::controllers::server_controller::ServerController;
use crate::controllers::user_controller::UserController;
use crate::cookie::auth_cookie::AuthCookie;
use crate::token::token_service::TokenService;
use portmaster_app::{Clock, Logger, LoggerFactory, SortableIdGenerator};
use std::time::Duration;

/// Os factories dos controllers e do que a pilha de middlewares consome.
///
/// Fecha o mesmo padrão das camadas de baixo aqui em cima: cada método devolve
/// `impl Trait` — contrato, nunca tipo concreto — e o grafo inteiro é
/// monomorfizado.
///
/// ## Os controllers são construídos uma vez
///
/// Ao contrário dos casos de uso, que são baratos de reconstruir a cada chamada,
/// os controllers nascem no boot e são **clonados** por requisição. É o que o
/// [`register`](crate::register::register) faz: ele consome o `AppProvider`, tira
/// dele tudo que as rotas precisam, e o descarta. Depois do boot não existe
/// provider em memória, nem `Arc` para mantê-lo vivo — só os controllers que as
/// rotas seguram, cada um um punhado de handles.
pub(crate) trait ApiProvider {
    /// A conta do próprio usuário.
    fn account_controller(&self) -> impl AccountController + use<Self> + 'static;

    /// Sessão: login, setup, refresh e logout.
    fn auth_controller(&self) -> impl AuthController + use<Self> + 'static;

    /// Contêineres.
    fn container_controller(&self) -> impl ContainerController + use<Self> + 'static;

    /// Carga e telemetria.
    fn manifest_controller(&self) -> impl ManifestController + use<Self> + 'static;

    /// Metadados de sistema.
    fn metadata_controller(&self) -> impl MetadataController + use<Self> + 'static;

    /// O painel do pátio.
    fn metrics_controller(&self) -> impl MetricsController + use<Self> + 'static;

    /// Produtos.
    fn product_controller(&self) -> impl ProductController + use<Self> + 'static;

    /// Papéis.
    fn role_controller(&self) -> impl RoleController + use<Self> + 'static;

    /// O estado do próprio processo.
    fn server_controller(&self) -> impl ServerController + use<Self> + 'static;

    /// Usuários.
    fn user_controller(&self) -> impl UserController + use<Self> + 'static;

    /// Quem emite e confere o access token, para o middleware de sessão.
    fn token_service(&self) -> impl TokenService + use<Self> + 'static;

    /// Como os cookies de sessão são lidos, para o middleware de sessão.
    fn auth_cookie(&self) -> impl AuthCookie + use<Self> + 'static;

    /// Fábrica de loggers, para os middlewares.
    fn logger_factory(&self) -> impl LoggerFactory + use<Self> + 'static;

    /// Um logger nomeado, para quem não é middleware.
    fn logger(&self, name: &str) -> impl Logger + use<Self> + 'static;

    /// Gerador de id ordenável, para o `request_id`.
    fn sortable_id_generator(&self) -> impl SortableIdGenerator + use<Self> + 'static;

    /// A hora corrente, para a latência que o log registra.
    fn clock(&self) -> impl Clock + use<Self> + 'static;

    /// Por quanto tempo uma requisição pode demorar.
    fn request_timeout(&self) -> Duration;

    /// As origens que o CORS libera.
    ///
    /// Vazio quando a API e o front saem do mesmo host, que é o padrão seguro.
    fn cors_origins(&self) -> &[String];
}
