//! O middleware que carimba de onde a resposta veio.

use std::task::{Context, Poll};

use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue};
use axum::response::Response;
use futures::future::BoxFuture;
use portmaster_app::{MetaEvent, MetaEventStackSubscriber};
use tower::{Layer, Service};

/// O cabeçalho da RFC 9211.
const CACHE_STATUS_HEADER: HeaderName = HeaderName::from_static("cache-status");

/// O valor quando a View saiu do cache de leitura.
///
/// O `portmaster` da frente é como este cache se identifica. A RFC pede que cada
/// cache da cadeia se nomeie, porque a resposta pode atravessar vários — um CDN
/// e um proxy acrescentam os deles ao mesmo campo —, e é o nome que distingue um
/// acerto **nosso** de um acerto de quem está na frente.
///
/// Literal e não montado a partir de uma `const` de nome: `from_static` exige
/// uma string literal, e é ela que dispensa a validação em execução.
const HIT: HeaderValue = HeaderValue::from_static("portmaster; hit");

/// O valor quando ela veio do banco.
const MISS: HeaderValue = HeaderValue::from_static("portmaster; fwd=miss");

/// Publica na resposta se ela foi servida do cache de leitura.
///
/// ## Ele não sabe o que é cache
///
/// Não conhece o `ViewCacheRepository`, não conhece grupo nem chave, e não
/// consegue descobrir sozinho se houve acerto. O que ele faz é perguntar à pilha
/// de eventos da tarefa se o
/// [`ViewCacheHit`](portmaster_app::MetaEvent::ViewCacheHit) foi registrado — e
/// quem o registra é o caso de uso, três camadas abaixo, sem saber que este
/// arquivo existe.
///
/// ## Os dois valores, e não só o acerto
///
/// Marcar apenas o acerto tornaria a ausência do cabeçalho ambígua: ela
/// significaria "veio do banco" e também "este layer saiu da pilha" ou "o escopo
/// não foi aberto". Dizer `fwd=miss` separa as duas — um teste que afirma
/// `fwd=miss` está afirmando que o caminho todo funciona e deu erro no cache, e
/// não que o mecanismo sumiu.
#[derive(Clone)]
pub(crate) struct CacheStatusLayer<E, S = ()> {
    /// O serviço interno, que este envolve; `()` enquanto é só layer.
    inner: S,
    /// A pilha a quem a pergunta é feita.
    events: E,
}

impl<E> CacheStatusLayer<E> {
    /// Monta o layer com a pilha que o provider entregou.
    pub(crate) const fn new(events: E) -> Self {
        Self { inner: (), events }
    }
}

impl<S, E: Clone> Layer<S> for CacheStatusLayer<E> {
    type Service = CacheStatusLayer<E, S>;

    fn layer(&self, inner: S) -> Self::Service {
        CacheStatusLayer {
            inner,
            events: self.events.clone(),
        }
    }
}

impl<S, E> Service<Request> for CacheStatusLayer<E, S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    E: MetaEventStackSubscriber + Clone + Send + Sync + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    /// A pergunta é feita **depois** da resposta, e dentro do escopo.
    ///
    /// O caso de uso emite durante o `call` do serviço interno, então perguntar
    /// antes responderia sempre `false`. E o escopo é aberto por um layer mais
    /// externo — ver [`MetaEventLayer`](super::meta_event_layer::MetaEventLayer).
    fn call(&mut self, request: Request) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let events = self.events.clone();

        Box::pin(async move {
            let mut response = inner.call(request).await?;

            let status = if events.captured(MetaEvent::ViewCacheHit) {
                HIT
            } else {
                MISS
            };

            response.headers_mut().insert(CACHE_STATUS_HEADER, status);

            Ok(response)
        })
    }
}
