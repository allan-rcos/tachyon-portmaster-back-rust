//! O middleware que desiste depois do prazo.

use std::task::{Context, Poll};
use std::time::Duration;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use futures::future::BoxFuture;
use portmaster_app::{Logger as _, SystemLogger};
use tower::{Layer, Service};

use crate::middleware::encode_port::EncodePort as _;
use crate::middleware::intern::encode_context::EncodeContext;
use crate::ports::error::api_error::ApiError;

/// Desiste da requisição depois do prazo.
///
/// ## O layer é o serviço antes de saber o que envolve
///
/// Um tipo só, e não um par. `TimeoutLayer` sem parâmetro é o `Layer` — o teto
/// configurado, ainda sem serviço interno —, e `TimeoutLayer<S>` é o `Service`
/// que sai do `layer()`: o mesmo teto, agora com o `S` que ele embrulha. Eram
/// dois tipos em dois arquivos, e o segundo nunca foi nomeado por ninguém — quem
/// monta a pilha aplica o layer e mais nada.
#[derive(Clone, Copy)]
pub(crate) struct TimeoutLayer<S = ()> {
    /// O serviço interno, que este envolve; `()` enquanto é só layer.
    inner: S,
    /// Teto de tempo de uma requisição.
    limit: Duration,
}

impl TimeoutLayer {
    /// Monta o layer com o teto configurado.
    pub(crate) const fn new(limit: Duration) -> Self {
        Self { inner: (), limit }
    }
}

impl<S> Layer<S> for TimeoutLayer {
    type Service = TimeoutLayer<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TimeoutLayer {
            inner,
            limit: self.limit,
        }
    }
}

impl<S> Service<Request> for TimeoutLayer<S>
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

    fn call(&mut self, request: Request) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let limit = self.limit;

        Box::pin(async move {
            tokio::select! {
                biased;

                response = inner.call(request) => response,

                () = tokio::time::sleep(limit) => {
                    SystemLogger::get()
                        .warn(
                            "requisição excedeu o teto de tempo",
                            [("limit_ms", &limit.as_millis().to_string())],
                        );

                    let (status, problem) = ApiError::new(
                        StatusCode::GATEWAY_TIMEOUT,
                        "The request took too long to complete.",
                    )
                    .into_parts();

                    Ok(EncodeContext.respond(status, &problem))
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse as _;
    use tower::{ServiceBuilder, ServiceExt};

    #[tokio::test]
    async fn o_que_demora_demais_vira_504() {
        let service = ServiceBuilder::new()
            .layer(TimeoutLayer::new(Duration::from_millis(10)))
            .service_fn(|_: Request| async move {
                tokio::time::sleep(Duration::from_secs(30)).await;
                Ok::<_, std::convert::Infallible>(Response::new(axum::body::Body::empty()))
            });

        let response = service
            .oneshot(Request::new(axum::body::Body::empty()))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
    }

    #[tokio::test]
    async fn o_que_responde_a_tempo_atravessa() {
        let service = ServiceBuilder::new()
            .layer(TimeoutLayer::new(Duration::from_secs(30)))
            .service_fn(|_: Request| async move {
                Ok::<_, std::convert::Infallible>((StatusCode::CREATED, "rápido").into_response())
            });

        let response = service
            .oneshot(Request::new(axum::body::Body::empty()))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }
}
