//! O teto de tempo de uma requisição.
//!
//! Sem ele, uma consulta que trava segura uma conexão do pool até o banco
//! desistir — e o cliente, que já foi embora, deixa para trás um recurso que
//! ninguém libera. Sob carga, é assim que um endpoint lento derruba os outros.
//!
//! O `504` é honesto: a requisição não foi recusada nem falhou, ela **não
//! terminou a tempo**. O trabalho em andamento é abandonado quando o futuro é
//! descartado, e a transação cai por conta do `Drop` do `sqlx`.

use std::task::{Context, Poll};
use std::time::Duration;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures::future::BoxFuture;
use tower::{Layer, Service};

/// Aplica o [`Timeout`].
#[derive(Clone, Copy)]
pub(crate) struct TimeoutLayer {
    limit: Duration,
}

impl TimeoutLayer {
    /// Monta o layer com o teto configurado.
    pub(crate) fn new(limit: Duration) -> Self {
        Self { limit }
    }
}

impl<S> Layer<S> for TimeoutLayer {
    type Service = Timeout<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Timeout {
            inner,
            limit: self.limit,
        }
    }
}

/// O serviço que desiste depois do prazo.
#[derive(Clone)]
pub(crate) struct Timeout<S> {
    inner: S,
    limit: Duration,
}

impl<S> Service<Request> for Timeout<S>
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
                // `biased` faz o `select!` conferir a resposta antes do prazo:
                // sem isso, uma requisição que terminou no mesmo instante em que
                // o timer disparou seria decidida por sorteio.
                biased;

                response = inner.call(request) => response,

                () = tokio::time::sleep(limit) => {
                    tracing::warn!(?limit, "requisição excedeu o teto de tempo");

                    Ok(StatusCode::GATEWAY_TIMEOUT.into_response())
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
