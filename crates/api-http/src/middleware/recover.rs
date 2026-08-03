//! A última defesa contra pânico dentro de uma requisição.
//!
//! A defesa é em três camadas: a `infra` **previne** (embrulha chamada a lib
//! externa e converte pânico em `anyhow`), este middleware **evita a queda**, e
//! o `panic::set_hook` do `main` **loga** o que escapar dos dois.
//!
//! O que ele compra: um pânico num handler derruba **aquela** requisição com
//! 500, e não o servidor inteiro. Sem ele, o `tokio` aborta a task e o axum
//! devolve uma conexão fechada sem resposta — o cliente vê um erro de rede e
//! ninguém fica sabendo o motivo.

use std::panic::AssertUnwindSafe;
use std::task::{Context, Poll};

use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures::future::{BoxFuture, FutureExt};
use tower::{Layer, Service};

/// Aplica o [`Recover`].
#[derive(Clone, Copy, Default)]
pub(crate) struct RecoverLayer;

impl RecoverLayer {
    /// Monta o layer.
    pub(crate) fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for RecoverLayer {
    type Service = Recover<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Recover { inner }
    }
}

/// O serviço que captura pânico.
#[derive(Clone)]
pub(crate) struct Recover<S> {
    inner: S,
}

impl<S> Service<Request> for Recover<S>
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
        // O clone é o padrão do tower: `poll_ready` foi chamado neste `self`, e
        // é este que está pronto — o original fica para a próxima requisição.
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            // `AssertUnwindSafe` porque o futuro do handler não é
            // `UnwindSafe` — quase nenhum é, já que captura `&mut`. A afirmação
            // se sustenta porque o que sobra de um pânico aqui é descartado: a
            // requisição termina, e nada do estado tocado por ela é lido depois.
            let outcome = AssertUnwindSafe(inner.call(request)).catch_unwind().await;

            Ok(match outcome {
                Ok(response) => response?,
                Err(panic) => {
                    tracing::error!(panic = %describe(&panic), "pânico capturado no handler");

                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        [(axum::http::header::CONTENT_TYPE, "application/problem+json")],
                        br#"{"type":"about:blank","title":"Internal Server Error","status":500,"detail":"An unexpected error occurred."}"#.to_vec(),
                    )
                        .into_response()
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

#[cfg(test)]
mod tests {
    use super::*;
    use tower::{ServiceBuilder, ServiceExt};

    #[tokio::test]
    async fn um_panico_vira_500_e_nao_derruba_o_servidor() {
        let service =
            ServiceBuilder::new()
                .layer(RecoverLayer::new())
                .service_fn(|_: Request| async move {
                    panic!("algo explodiu no handler");
                    #[allow(unreachable_code)]
                    Ok::<_, std::convert::Infallible>(Response::new(axum::body::Body::empty()))
                });

        let response = service
            .oneshot(Request::new(axum::body::Body::empty()))
            .await
            .expect("o middleware não deveria propagar o pânico");

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn a_resposta_normal_atravessa_intacta() {
        let service =
            ServiceBuilder::new()
                .layer(RecoverLayer::new())
                .service_fn(|_: Request| async move {
                    Ok::<_, std::convert::Infallible>(
                        (StatusCode::CREATED, "tudo certo").into_response(),
                    )
                });

        let response = service
            .oneshot(Request::new(axum::body::Body::empty()))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }
}
