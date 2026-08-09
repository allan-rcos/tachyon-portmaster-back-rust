//! O serviço de log de requisição.

use std::task::{Context, Poll};

use axum::extract::Request;
use axum::response::Response;
use futures::future::BoxFuture;
use portmaster_app::{Clock, Logger};
use tower::Service;

use super::request_id_header::REQUEST_ID_HEADER;

/// O nome do componente nos logs.
pub(super) const CHANNEL: &str = "http";

/// O serviço que registra a requisição.
#[derive(Clone)]
pub(crate) struct Logging<S, L, K> {
    /// O serviço interno, que este envolve.
    pub(super) inner: S,
    /// O logger desta requisição, já com o id carimbado.
    pub(super) logger: L,
    /// De onde saem os dois instantes que viram latência.
    pub(super) clock: K,
}

impl<S, L, K> Service<Request> for Logging<S, L, K>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    L: Logger,
    K: Clock,
{
    type Response = Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    /// Mede a latência e emite a linha com o status final.
    ///
    /// A medição sai do [`Clock`] injetado, e não de um `Instant::now` com
    /// `allow` em cima. Perde-se a monotonicidade — um ajuste de NTP no meio de
    /// uma resposta produziria um `duration_ms` esquisito —, e isso é um evento
    /// que não acontece dentro de uma resposta de milissegundos. Ganha-se um
    /// relógio só no sistema, e nenhuma exceção de linter para justificar.
    fn call(&mut self, request: Request) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let method = request.method().to_string();
        let path = request.uri().path().to_owned();
        let request_id = request
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();

        let logger = self
            .logger
            .with_field("request_id", request_id)
            .with_field("method", method)
            .with_field("path", path);

        let clock = self.clock.clone();
        let started = clock.now();

        Box::pin(async move {
            let response = inner.call(request).await?;

            let elapsed = clock.now().signed_duration_since(started).num_milliseconds();
            let logger = logger
                .with_field("status", response.status().as_u16().to_string())
                .with_field("duration_ms", elapsed.to_string());

            if response.status().is_server_error() {
                logger.error("requisição falhou");
            } else {
                logger.info("requisição atendida");
            }

            Ok(response)
        })
    }
}
