//! O middleware que transforma pânico em resposta.

use std::panic::AssertUnwindSafe;
use std::task::{Context, Poll};

use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use futures::future::{BoxFuture, FutureExt};
use portmaster_app::{Logger as _, SystemLogger};
use tower::{Layer, Service};

use crate::middleware::encode_port::EncodePort as _;
use crate::middleware::intern::encode_context::EncodeContext;
use crate::ports::error::api_error::ApiError;

/// Transforma um pânico do handler em resposta.
///
/// ## O layer é o serviço antes de saber o que envolve
///
/// Um tipo só, e não um par. `RecoverLayer` sem parâmetro é o `Layer`, e
/// `RecoverLayer<S>` é o `Service` que sai do `layer()` — o mesmo tipo, agora
/// com o `S` que ele embrulha. Um par de tipos custaria um segundo arquivo e um
/// nome que ninguém pronuncia: quem monta a pilha aplica o layer e mais nada.
#[derive(Clone, Copy, Default)]
pub(crate) struct RecoverLayer<S = ()> {
    /// O serviço interno, que este envolve; `()` enquanto é só layer.
    inner: S,
}

impl RecoverLayer {
    /// Monta o layer.
    pub(crate) const fn new() -> Self {
        Self { inner: () }
    }
}

impl<S> Layer<S> for RecoverLayer {
    type Service = RecoverLayer<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RecoverLayer { inner }
    }
}

impl<S> Service<Request> for RecoverLayer<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    /// Chama o serviço interno e transforma um pânico em resposta.
    ///
    /// O corpo de `500` sai pela
    /// [`EncodePort`](crate::middleware::encode_port::EncodePort), no formato
    /// que a requisição negociou. Este layer não relê o `Accept`: quem o
    /// resolveu foi o `EncodeLayer`, que por isso tem de estar **fora** deste na
    /// pilha. Cada middleware que responde por conta própria remontar a
    /// negociação seria a mesma decisão tomada em três lugares.
    ///
    /// Um literal de bytes JSON aqui seria pior do que parece: num sistema cujo
    /// cliente de produção fala `FlatBuffers`, é o único corpo que ele nunca
    /// conseguiria ler.
    fn call(&mut self, request: Request) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let outcome = AssertUnwindSafe(inner.call(request)).catch_unwind().await;

            Ok(match outcome {
                Ok(response) => response?,
                Err(panic) => {
                    SystemLogger::get().error(
                        "pânico capturado no controller",
                        [("panic", &describe(&panic))],
                    );

                    let (status, problem) = ApiError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "An unexpected error occurred.",
                    )
                    .into_parts();

                    EncodeContext.respond(status, &problem)
                }
            })
        })
    }
}

/// O que dá para dizer sobre o payload de um pânico.
fn describe(panic: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = panic.downcast_ref::<&str>() {
        return (*message).to_owned();
    }

    if let Some(message) = panic.downcast_ref::<String>() {
        return message.clone();
    }

    "pânico sem mensagem legível".to_owned()
}
