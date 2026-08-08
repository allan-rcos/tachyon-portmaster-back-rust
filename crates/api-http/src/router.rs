//! A tabela de rotas e a pilha de middlewares.
//!
//! É o único lugar onde uma URL aparece. Tudo que o servidor atende está aqui, e
//! nada aqui decide nada — cada rota aponta para um método de handler, e o
//! handler aponta para um caso de uso.
//!
//! ## Por que a função é genérica sobre o provider
//!
//! Os casos de uso que o `AppProvider` entrega têm tipos **innomeáveis**: só
//! existem depois da monomorfização. Nenhuma struct consegue declarar o tipo do
//! que recebe, então tudo daqui para baixo é genérico — os handlers sobre os
//! casos de uso, esta função sobre o provider. É o preço da DI 100% estática, e
//! o que ela compra é que não há `dyn` nem container em lugar nenhum do wiring.
//!
//! ## Os casos de uso nascem por requisição
//!
//! Cada closure de rota constrói o handler no momento em que a requisição chega.
//! Isso é barato de propósito: um caso de uso não guarda estado, e o que custa
//! caro — pool de conexões, caches — nasceu uma vez no `register` e chega por
//! clone. Construí-los uma vez só exigiria que o provider fosse `'static`, o que
//! na prática significa vazá-lo — mais complexidade do que a alocação que se
//! evitaria.
//!
//! ## A ordem da pilha
//!
//! Os `.layer()` do axum aplicam-se **de baixo para cima**: o último declarado é
//! o mais externo. A ordem abaixo é lida de fora para dentro assim:
//!
//! 1. **`RequestId`** — carimba antes de tudo, para que a linha de log tenha id.
//! 2. **Logging** — vê o status final, inclusive o 500 que o Recover produziu.
//! 3. **Recover** — um pânico vira 500 desta requisição, não queda do processo.
//! 4. **Timeout** — o teto vale para o handler, não para o log em volta dele.
//! 5. **CORS** — responde ao preflight sem incomodar a sessão.
//! 6. **Token** — o mais interno, porque o escopo de sessão precisa cobrir o
//!    handler e nada mais.

use std::sync::Arc;
use std::time::Duration;

use axum::routing::{get, post, put};
use axum::Router;
use portmaster_app::AppProvider;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::config::api_config::ApiConfig;
use crate::cookie::AuthCookie;
use crate::handlers::account::AccountHandlers;
use crate::handlers::auth::AuthHandlers;
use crate::handlers::container::ContainerHandlers;
use crate::handlers::manifest::ManifestHandlers;
use crate::handlers::metadata::MetadataHandlers;
use crate::handlers::metrics::MetricsHandlers;
use crate::handlers::product::ProductHandlers;
use crate::handlers::role::RoleHandlers;
use crate::handlers::server::ServerHandlers;
use crate::handlers::user::UserHandlers;
use crate::middleware::logging_layer::LoggingLayer;
use crate::middleware::negotiation_layer::NegotiationLayer;
use crate::middleware::recover_layer::RecoverLayer;
use crate::middleware::request_id_layer::RequestIdLayer;
use crate::middleware::timeout_layer::TimeoutLayer;
use crate::middleware::token_layer::TokenLayer;
use crate::token::token_service::TokenService;

/// Liga uma rota ao método de um handler de recurso.
///
/// Monta o handler a partir do factory do provider e chama o método com os
/// extractors que a closure recebeu. Os tipos dos extractors são escritos na
/// chamada porque o axum precisa deles para escolher a implementação de
/// `Handler` — inferi-los não é possível de dentro da closure.
macro_rules! route {
    ($provider:expr, $factory:ident => $handlers:ident::$method:ident $(, $arg:ident: $ty:ty)* $(,)?) => {{
        let provider = ::std::sync::Arc::clone(&$provider);

        move |$($arg: $ty),*| {
            let provider = ::std::sync::Arc::clone(&provider);

            async move {
                $handlers::new(provider.$factory())
                    .$method($($arg),*)
                    .await
            }
        }
    }};
}

/// Liga uma rota ao método dos handlers de sessão.
///
/// Separado do [`route!`] porque `AuthHandlers` é o único que precisa de mais do
/// que um factory: ele carrega token, cookies e o prazo do refresh, que são
/// desta camada e não do `app`.
macro_rules! auth_route {
    ($provider:expr, $auth:expr, $method:ident $(, $arg:ident: $ty:ty)* $(,)?) => {{
        let provider = ::std::sync::Arc::clone(&$provider);
        let parts = $auth.clone();

        move |$($arg: $ty),*| {
            let provider = ::std::sync::Arc::clone(&provider);
            let parts = parts.clone();

            async move {
                AuthHandlers::new(
                    provider.session_use_case(),
                    provider.mark_use_case(),
                    provider.random_id_generator(),
                    parts.tokens,
                    parts.cookies,
                    parts.refresh_ttl_seconds,
                )
                .$method($($arg),*)
                .await
            }
        }
    }};
}

/// O que os handlers de sessão precisam além do provider.
#[derive(Clone)]
struct AuthParts {
    tokens: TokenService,
    cookies: AuthCookie,
    refresh_ttl_seconds: u64,
}

/// Monta o router inteiro sobre um provider.
#[allow(
    clippy::too_many_lines,
    reason = "é a tabela de rotas: quebrá-la em pedaços esconderia o mapa que o arquivo existe para mostrar"
)]
#[allow(
    clippy::needless_pass_by_value,
    reason = "o router toma posse do provider e da config: eles vivem enquanto o servidor viver"
)]
pub fn router<P: AppProvider + Send + Sync + 'static>(
    provider: Arc<P>,
    config: ApiConfig,
) -> Router {
    use crate::handlers::params::container_page_params::ContainerPageParams;
    use crate::handlers::params::page_params::PageParams;
    use crate::handlers::params::search_params::SearchParams;
    use crate::handlers::params::summary_page_params::SummaryPageParams;
    use crate::handlers::params::user_page_params::UserPageParams;
    use crate::wire::body::Body;
    use crate::wire::dto::account::account_update_request_factory::AccountUpdateRequestFactory;
    use crate::wire::dto::account::password_change_request_factory::PasswordChangeRequestFactory;
    use crate::wire::dto::admin::role_create_request_factory::RoleCreateRequestFactory;
    use crate::wire::dto::admin::role_ids_request::RoleIdsRequest;
    use crate::wire::dto::admin::role_permissions_update_request_factory::RolePermissionsUpdateRequestFactory;
    use crate::wire::dto::admin::user_create_request_factory::UserCreateRequestFactory;
    use crate::wire::dto::admin::user_password_reset_request_factory::UserPasswordResetRequestFactory;
    use crate::wire::dto::admin::user_update_request_factory::UserUpdateRequestFactory;
    use crate::wire::dto::auth::login_request_factory::LoginRequestFactory;
    use crate::wire::dto::auth::setup_request_factory::SetupRequestFactory;
    use crate::wire::dto::container::container_create_request_factory::ContainerCreateRequestFactory;
    use crate::wire::dto::container::container_update_request_factory::ContainerUpdateRequestFactory;
    use crate::wire::dto::manifest::load_item_request_factory::LoadItemRequestFactory;
    use crate::wire::dto::manifest::unload_item_request_factory::UnloadItemRequestFactory;
    use crate::wire::dto::product::product_create_request_factory::ProductCreateRequestFactory;
    use crate::wire::dto::product::product_update_request_factory::ProductUpdateRequestFactory;
    use crate::wire::json_body::JsonBody;
    use crate::wire::wire::Wire;
    use axum::extract::{Path, Query};
    use axum::http::HeaderMap;

    let tokens = TokenService::new(&config.jwt);
    let cookies = AuthCookie::new(&config.jwt);
    let auth = AuthParts {
        tokens: tokens.clone(),
        cookies: cookies.clone(),
        refresh_ttl_seconds: config.jwt.refresh_ttl.as_secs(),
    };
    let environment = config.environment.clone();

    Router::new()
        // --- Introspecção (pública) -------------------------------------
        .route(
            "/info",
            get({
                let handlers = Arc::new(ServerHandlers::new(environment));

                move |wire: Wire| {
                    let handlers = Arc::clone(&handlers);

                    async move { handlers.info(wire).await }
                }
            }),
        )
        // --- Sessão (pública) -------------------------------------------
        .route(
            "/setup",
            post(auth_route!(
                provider,
                auth,
                setup,
                wire: Wire,
                body: Body<SetupRequestFactory>,
            )),
        )
        .route(
            "/auth/login",
            post(auth_route!(
                provider,
                auth,
                login,
                wire: Wire,
                body: Body<LoginRequestFactory>,
            )),
        )
        .route(
            "/auth/refresh",
            post(auth_route!(provider, auth, refresh, headers: HeaderMap)),
        )
        .route(
            "/auth/logout",
            post(auth_route!(provider, auth, logout, headers: HeaderMap)),
        )
        // --- Conta própria ----------------------------------------------
        .route(
            "/account",
            get(route!(provider, account_use_case => AccountHandlers::get, wire: Wire)).put(
                route!(
                    provider,
                    account_use_case => AccountHandlers::update,
                    wire: Wire,
                    body: Body<AccountUpdateRequestFactory>,
                ),
            ),
        )
        .route(
            "/account/password",
            put(route!(
                provider,
                account_use_case => AccountHandlers::change_password,
                body: Body<PasswordChangeRequestFactory>,
            )),
        )
        // --- Produtos ----------------------------------------------------
        .route(
            "/products",
            get(route!(
                provider,
                product_use_case => ProductHandlers::list,
                wire: Wire,
                params: Query<PageParams>,
            ))
            .post(route!(
                provider,
                product_use_case => ProductHandlers::create,
                wire: Wire,
                body: Body<ProductCreateRequestFactory>,
            )),
        )
        .route(
            "/products/{id}",
            get(route!(
                provider,
                product_use_case => ProductHandlers::get,
                wire: Wire,
                id: Path<String>,
            ))
            .put(route!(
                provider,
                product_use_case => ProductHandlers::update,
                wire: Wire,
                id: Path<String>,
                body: Body<ProductUpdateRequestFactory>,
            ))
            .delete(route!(
                provider,
                product_use_case => ProductHandlers::delete,
                id: Path<String>,
            )),
        )
        // --- Papéis -------------------------------------------------------
        .route(
            "/roles",
            get(route!(
                provider,
                role_use_case => RoleHandlers::list,
                wire: Wire,
                params: Query<PageParams>,
            ))
            .post(route!(
                provider,
                role_use_case => RoleHandlers::create,
                wire: Wire,
                body: Body<RoleCreateRequestFactory>,
            )),
        )
        .route(
            "/roles/{id}/permissions",
            put(route!(
                provider,
                role_use_case => RoleHandlers::update_permissions,
                wire: Wire,
                id: Path<String>,
                body: Body<RolePermissionsUpdateRequestFactory>,
            )),
        )
        // --- Contêineres ---------------------------------------------------
        .route(
            "/containers",
            get(route!(
                provider,
                container_use_case => ContainerHandlers::list,
                wire: Wire,
                params: Query<ContainerPageParams>,
            ))
            .post(route!(
                provider,
                container_use_case => ContainerHandlers::create,
                wire: Wire,
                body: Body<ContainerCreateRequestFactory>,
            )),
        )
        // Antes de `/containers/{id}`: um literal e um parâmetro no mesmo
        // segmento são ambíguos, e o axum resolve pelo literal — mas a ordem
        // aqui deixa a intenção legível para quem vier acrescentar rota.
        .route(
            "/containers/summary",
            get(route!(
                provider,
                container_use_case => ContainerHandlers::summary,
                wire: Wire,
                params: Query<SummaryPageParams>,
            )),
        )
        .route(
            "/containers/{id}",
            get(route!(
                provider,
                container_use_case => ContainerHandlers::get,
                wire: Wire,
                id: Path<String>,
            ))
            .put(route!(
                provider,
                container_use_case => ContainerHandlers::update,
                wire: Wire,
                id: Path<String>,
                body: Body<ContainerUpdateRequestFactory>,
            ))
            .delete(route!(
                provider,
                container_use_case => ContainerHandlers::delete,
                id: Path<String>,
            )),
        )
        .route(
            "/containers/{id}/seal",
            post(route!(
                provider,
                container_use_case => ContainerHandlers::seal,
                id: Path<String>,
            )),
        )
        .route(
            "/containers/{id}/dispatch",
            post(route!(
                provider,
                container_use_case => ContainerHandlers::dispatch,
                id: Path<String>,
            )),
        )
        // --- Manifestos -----------------------------------------------------
        .route(
            "/manifests/load-item",
            post(route!(
                provider,
                manifest_use_case => ManifestHandlers::load,
                wire: Wire,
                body: Body<LoadItemRequestFactory>,
            )),
        )
        .route(
            "/manifests/unload-item",
            post(route!(
                provider,
                manifest_use_case => ManifestHandlers::unload,
                wire: Wire,
                body: Body<UnloadItemRequestFactory>,
            )),
        )
        // --- Usuários --------------------------------------------------------
        .route(
            "/users",
            get(route!(
                provider,
                user_use_case => UserHandlers::list,
                wire: Wire,
                params: Query<UserPageParams>,
            ))
            .post(route!(
                provider,
                user_use_case => UserHandlers::create,
                wire: Wire,
                body: Body<UserCreateRequestFactory>,
            )),
        )
        .route(
            "/users/{id}",
            get(route!(
                provider,
                user_use_case => UserHandlers::get,
                wire: Wire,
                id: Path<String>,
            ))
            .put(route!(
                provider,
                user_use_case => UserHandlers::update,
                wire: Wire,
                id: Path<String>,
                body: Body<UserUpdateRequestFactory>,
            ))
            .delete(route!(
                provider,
                user_use_case => UserHandlers::delete,
                id: Path<String>,
            )),
        )
        .route(
            "/users/{id}/password",
            put(route!(
                provider,
                user_use_case => UserHandlers::reset_password,
                id: Path<String>,
                body: Body<UserPasswordResetRequestFactory>,
            )),
        )
        .route(
            "/users/{id}/roles",
            put(route!(
                provider,
                user_use_case => UserHandlers::update_roles,
                wire: Wire,
                id: Path<String>,
                body: JsonBody<RoleIdsRequest>,
            )),
        )
        // --- Metadados e métricas ---------------------------------------------
        .route(
            "/metadata/permissions",
            get(route!(
                provider,
                metadata_use_case => MetadataHandlers::list_permissions,
                wire: Wire,
                params: Query<SearchParams>,
            )),
        )
        .route(
            "/metrics",
            get(route!(
                provider,
                metrics_use_case => MetricsHandlers::get,
                wire: Wire,
            )),
        )
        // --- A pilha, de dentro para fora --------------------------------------
        //
        // A negociação vem **imediatamente acima** do token: ela precisa estar
        // instalada antes de qualquer handler extrair um `Wire`, e não depende
        // de nada que as camadas de fora façam. Reordená-la para depois faria
        // toda rota com `Wire` responder 500 — deliberado, e documentado no
        // `FromRequestParts` do `Wire`.
        .layer(TokenLayer::new(tokens, cookies))
        .layer(NegotiationLayer::new())
        .layer(cors_layer(&config.cors_origins))
        .layer(TimeoutLayer::new(config.request_timeout))
        .layer(RecoverLayer::new())
        .layer(LoggingLayer::new(provider.logger_factory()))
        .layer(RequestIdLayer::new(provider.sortable_id_generator()))
}

/// O CORS, quando há origem configurada.
///
/// Sem `APP_CORS_ORIGINS` o layer não acrescenta cabeçalho nenhum — é o estado
/// em que a API sempre operou, e ligar CORS por padrão mudaria a resposta de
/// todo mundo para atender a um cliente que talvez não exista.
///
/// Com origens configuradas, as credenciais são liberadas: a sessão vive num
/// cookie, e sem `Allow-Credentials` o navegador não o enviaria — o CORS estaria
/// ligado e a API continuaria inalcançável. Por isso a origem é sempre a lista
/// declarada, e nunca `*`: as duas coisas juntas são recusadas pelo navegador, e
/// com razão.
fn cors_layer(origins: &[String]) -> CorsLayer {
    if origins.is_empty() {
        return CorsLayer::new();
    }

    let allowed: Vec<_> = origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed))
        .allow_credentials(true)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
        .max_age(Duration::from_secs(3600))
}
