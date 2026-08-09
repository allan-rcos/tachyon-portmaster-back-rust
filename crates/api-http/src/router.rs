//! A composição das rotas e da pilha de middlewares.

use axum::http::{HeaderValue, Method};
use axum::Router;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

use crate::controllers::{
    account_routes, auth_routes, container_routes, manifest_routes, metadata_routes,
    metrics_routes, product_routes, role_routes, server_routes, user_routes,
};
use crate::middleware::logging_layer::LoggingLayer;
use crate::middleware::recover_layer::RecoverLayer;
use crate::middleware::request_id_layer::RequestIdLayer;
use crate::middleware::timeout_layer::TimeoutLayer;
use crate::middleware::token_layer::TokenLayer;
use crate::config::api_config::ApiConfig;
use crate::provider::ApiProvider;
use portmaster_app::AppProvider;

/// Por quanto tempo um preflight de CORS pode ser reaproveitado.
const CORS_MAX_AGE_SECONDS: u64 = 3600;

/// Monta o router completo.
///
/// Cada recurso traz as **próprias** rotas, e este arquivo só as junta. É o que
/// faz ele caber numa tela: o encanamento de extractor — qual rota tem corpo,
/// qual tem `Path`, qual mensagem cada corpo carrega — mora ao lado do
/// controller que o consome, e aqui não aparece nome nenhum de tipo interno.
///
/// Antes havia duas `macro_rules!` neste arquivo. Elas existiam para esconder
/// que cada rota reconstruía o seu controller a cada requisição, a partir de um
/// `Arc` do provider clonado duas vezes por chamada. Agora os controllers vêm
/// prontos do `ApiProvider`, são construídos **uma vez** aqui, e cada rota
/// clona o seu — um punhado de handles.
pub fn router<P: AppProvider>(app: P, config: ApiConfig) -> Router {
    routes(&crate::register::register(app, config))
}

/// Junta as rotas de cada recurso e aplica a pilha.
fn routes<P: ApiProvider>(provider: &P) -> Router {
    Router::new()
        .merge(server_routes::routes(provider.server_controller()))
        .merge(auth_routes::routes(provider.auth_controller()))
        .merge(account_routes::routes(provider.account_controller()))
        .merge(product_routes::routes(provider.product_controller()))
        .merge(role_routes::routes(provider.role_controller()))
        .merge(container_routes::routes(provider.container_controller()))
        .merge(manifest_routes::routes(provider.manifest_controller()))
        .merge(user_routes::routes(provider.user_controller()))
        .merge(metadata_routes::routes(provider.metadata_controller()))
        .merge(metrics_routes::routes(provider.metrics_controller()))
        // A pilha é aplicada de dentro para fora: o último `.layer` é o mais
        // externo, e por isso o `RequestId` vem por último — ele precisa
        // carimbar o id antes de o `Logging` procurá-lo.
        .layer(TokenLayer::new(
            provider.token_service(),
            provider.auth_cookie(),
        ))
        .layer(cors_layer(provider.cors_origins()))
        .layer(TimeoutLayer::new(provider.request_timeout()))
        .layer(RecoverLayer::new())
        .layer(LoggingLayer::new(
            provider.logger_factory(),
            provider.clock(),
        ))
        .layer(RequestIdLayer::new(provider.sortable_id_generator()))
}

/// A política de CORS.
///
/// Sem origens configuradas, a camada não libera nada — é o padrão seguro, e o
/// que vale quando a API e o front saem do mesmo host. Com origens, libera
/// credenciais: a sessão viaja em cookie, e sem `allow_credentials` o navegador
/// não o mandaria.
fn cors_layer(origins: &[String]) -> CorsLayer {
    if origins.is_empty() {
        return CorsLayer::new();
    }

    let allowed: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect();

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed))
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any)
        .max_age(std::time::Duration::from_secs(CORS_MAX_AGE_SECONDS))
}
