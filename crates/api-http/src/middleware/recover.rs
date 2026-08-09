//! O serviço de captura de pânico.

use std::panic::AssertUnwindSafe;
use std::task::{Context, Poll};

use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use futures::future::{BoxFuture, FutureExt};
use portmaster_app::{Logger as _, SystemLogger};
use tower::Service;

use crate::error::api_error::ApiError;
use crate::wire::encoder::Encoder;

/// O serviço que captura pânico.
#[derive(Clone)]
pub(crate) struct Recover<S> {
    /// O serviço interno, que este envolve.
    pub(super) inner: S,
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

    /// Chama o serviço interno e transforma um pânico em resposta.
    ///
    /// O encoder é montado **antes** da chamada, com os cabeçalhos da
    /// requisição: depois do pânico não há mais requisição de onde negociar, e o
    /// corpo de 500 precisa sair no formato que o cliente pediu como qualquer
    /// outro. O desenho anterior tinha aqui um literal de bytes JSON, que num
    /// sistema cujo cliente de produção fala `FlatBuffers` era o único corpo que
    /// ele nunca conseguiria ler.
    fn call(&mut self, request: Request) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let encoder = Encoder::of_headers(request.headers());

        Box::pin(async move {
            let outcome = AssertUnwindSafe(inner.call(request)).catch_unwind().await;

            Ok(match outcome {
                Ok(response) => response?,
                Err(panic) => {
                    SystemLogger::get()
                        .with_field("panic", describe(&panic))
                        .error("pânico capturado no controller");

                    let (status, problem, cookies) = ApiError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "An unexpected error occurred.",
                    )
                    .into_parts();

                    encoder.respond(status, &problem, cookies)
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
    use super::super::recover_layer::RecoverLayer;
    use super::*;
    use axum::response::IntoResponse as _;
    use tower::{ServiceBuilder, ServiceExt};

    #[tokio::test]
    async fn um_panico_vira_500_e_nao_derruba_o_servidor() {
        let service =
            ServiceBuilder::new()
                .layer(RecoverLayer::new())
                .service_fn(|_: Request| async move {
                    panic!("algo explodiu no handler");
                    #[allow(unreachable_code, reason = "o panic! acima é o assunto do teste")]
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
