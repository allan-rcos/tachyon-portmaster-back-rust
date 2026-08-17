//! Quem serve os middlewares da pilha.

use std::time::Duration;

use portmaster_app::AppProvider;

use crate::middleware::intern::cache_status_layer::CacheStatusLayer;
use crate::middleware::intern::cookie_layer::CookieLayer;
use crate::middleware::intern::decode_layer::DecodeLayer;
use crate::middleware::intern::encode_layer::EncodeLayer;
use crate::middleware::intern::logging_layer::LoggingLayer;
use crate::middleware::intern::meta_event_layer::MetaEventLayer;
use crate::middleware::intern::recover_layer::RecoverLayer;
use crate::middleware::intern::request_id_layer::RequestIdLayer;
use crate::middleware::intern::session_layer::SessionLayer;
use crate::middleware::intern::timeout_layer::TimeoutLayer;
use crate::ports::token::token_service::TokenService;
use crate::ports::token::TokenProvider;

/// Os layers da pilha, montados com o que cada um consome.
///
/// ## Por que estes não devolvem `impl Trait`
///
/// É a única exceção do grafo, e ela é do axum. O bound de `Router::layer` não
/// pede só `L: Layer<Route>`: ele pede coisas sobre `L::Service` — que seja
/// `Service<Request>`, `Clone`, `Send`, `'static`, e que a resposta dele vire
/// `IntoResponse`. Um `impl Layer<Route>` esconde `L::Service` atrás de um tipo
/// opaco, e não há como declarar bounds sobre o associado de um tipo opaco.
/// Devolver o tipo concreto é o que mantém a pilha composta por generics, sem
/// um `dyn` no meio dela.
///
/// Os tipos não saem do crate, então o nome de um layer alcança só o arquivo que
/// monta o router — que é justamente quem precisa conhecê-lo.
pub(crate) struct MiddlewareProvider;

impl MiddlewareProvider {
    /// Abre a sessão a partir do cookie de access.
    pub(crate) fn session() -> anyhow::Result<SessionLayer<impl TokenService + use<>>> {
        Ok(SessionLayer::new(TokenProvider::token_service()?))
    }

    /// Recolhe os cookies da requisição e publica os da resposta.
    pub(crate) const fn cookies() -> CookieLayer {
        CookieLayer::new()
    }

    /// Derruba a requisição que passar do teto de tempo.
    pub(crate) const fn timeout(limit: Duration) -> TimeoutLayer {
        TimeoutLayer::new(limit)
    }

    /// Transforma um pânico em resposta, em vez de queda.
    pub(crate) const fn recover() -> RecoverLayer {
        RecoverLayer::new()
    }

    /// Escolhe como o corpo da requisição é lido.
    pub(crate) const fn decode() -> DecodeLayer {
        DecodeLayer::new()
    }

    /// Escolhe em que formato a resposta sai.
    pub(crate) const fn encode() -> EncodeLayer {
        EncodeLayer::new()
    }

    /// Registra a requisição e o que ela respondeu.
    pub(crate) fn logging() -> LoggingLayer<impl portmaster_app::Logger> {
        LoggingLayer::new(&AppProvider::logger_factory())
    }

    /// Abre o escopo da pilha de eventos desta requisição.
    ///
    /// Precisa ser mais externo que todo layer que pergunte à pilha — hoje, o
    /// [`Self::cache_status`].
    pub(crate) fn meta_events(
    ) -> MetaEventLayer<impl portmaster_app::MetaEventStackSubscriber + Clone + use<>> {
        MetaEventLayer::new(AppProvider::meta_event_stack())
    }

    /// Publica se a resposta saiu do cache de leitura.
    pub(crate) fn cache_status(
    ) -> CacheStatusLayer<impl portmaster_app::MetaEventStackSubscriber + Clone + use<>> {
        CacheStatusLayer::new(AppProvider::meta_event_stack())
    }

    /// Abre o escopo com o `request_id` desta requisição.
    pub(crate) fn request_id() -> RequestIdLayer<impl portmaster_app::SequentialIdGenerator + use<>>
    {
        RequestIdLayer::new(AppProvider::sequential_id_generator())
    }
}
