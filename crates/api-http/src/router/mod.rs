//! A composição das versões e da pilha de middlewares.

use axum::http::{HeaderValue, Method};
use axum::Router;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

use crate::bootstrap::provider::ApiProvider;
use crate::config::api_config::ApiConfig;
use crate::config::jwt_config::JwtConfig;
use crate::controllers::auth_controller::AuthController;
use crate::middleware::intern::cookie_layer::CookieLayer;
use crate::middleware::intern::decode_layer::DecodeLayer;
use crate::middleware::intern::encode_layer::EncodeLayer;
use crate::middleware::intern::logging_layer::LoggingLayer;
use crate::middleware::intern::recover_layer::RecoverLayer;
use crate::middleware::intern::request_id_layer::RequestIdLayer;
use crate::middleware::intern::session_layer::SessionLayer;
use crate::middleware::intern::timeout_layer::TimeoutLayer;
use crate::router::router_hub::RouterHub;
use portmaster_app::AppProvider;

pub(crate) mod route;
pub(crate) mod router_hub;
pub(crate) mod versioned_router;

pub(crate) mod intern;

/// Por quanto tempo um preflight de CORS pode ser reaproveitado.
const CORS_MAX_AGE_SECONDS: u64 = 3600;

/// Monta o router completo.
///
/// As rotas vêm do `RouterHub`, que junta as versões publicadas; este arquivo
/// só acrescenta a pilha em volta. É o que faz ele caber numa tela.
pub async fn router<P: AppProvider>(
    app: P,
    config: ApiConfig,
    jwt: &JwtConfig,
) -> anyhow::Result<Router> {
    let provider = crate::bootstrap::register::register(app, config, jwt);

    // O grupo de marcador da sessão precisa existir antes da primeira
    // requisição, e quem sabe o nome dele é o controller que o usa.
    provider
        .auth_controller()
        .register_marker_group()
        .await
        .map_err(|error| anyhow::anyhow!("{error:?}"))?;

    let cors = cors_layer(provider.cors_origins());
    let timeout = provider.request_timeout();

    Ok(RouterHub::build(&provider)?
        // De dentro para fora: o último `.layer` é o mais externo.
        .layer(SessionLayer::new(provider.token_service()))
        .layer(CookieLayer::new())
        .layer(cors)
        .layer(TimeoutLayer::new(timeout))
        .layer(RecoverLayer::new())
        .layer(DecodeLayer::new())
        .layer(EncodeLayer::new())
        .layer(LoggingLayer::new(&provider.logger_factory()))
        .layer(RequestIdLayer::new(provider.sequential_id_generator())))
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
