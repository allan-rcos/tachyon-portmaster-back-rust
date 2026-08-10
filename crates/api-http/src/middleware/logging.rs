//! O serviço de log de requisição.

use std::task::{Context, Poll};

use axum::extract::Request;
use axum::response::Response;
use chrono::Utc;
use futures::future::BoxFuture;
use portmaster_app::Logger;
use tower::Service;
use tracing::Instrument as _;

use super::request_id_header::REQUEST_ID_HEADER;

/// O nome do componente nos logs.
pub(super) const CHANNEL: &str = "http";

/// O nome do span que envolve a requisição inteira.
const SPAN: &str = "request";

/// O serviço que registra a requisição.
#[derive(Clone)]
pub(crate) struct Logging<S, L> {
    /// O serviço interno, que este envolve.
    pub(super) inner: S,
    /// O logger do canal HTTP, sem nada de uma requisição em particular.
    pub(super) logger: L,
}

impl<S, L> Service<Request> for Logging<S, L>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    L: Logger,
{
    type Response = Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    /// Abre o span da requisição, mede a latência e emite a linha do desfecho.
    ///
    /// O span é o que faz o `request_id` alcançar quem não o recebeu: o resto da
    /// pilha corre dentro dele, então uma linha emitida lá no fundo do `app` ou
    /// da `infra` sai correlacionada sem que ninguém no meio do caminho tenha
    /// tocado no assunto. Carimbar o id num logger só alcançava quem tivesse
    /// aquele logger em mãos, e ninguém abaixo do middleware tinha.
    ///
    /// A latência sai de dois `Utc::now()`, e não de um `Instant::now` — que o
    /// `.clippy.toml` proíbe. Perde-se a monotonicidade: um ajuste de NTP no
    /// meio de uma resposta produziria um `duration_ms` esquisito, e isso é um
    /// evento que não acontece dentro de uma resposta de milissegundos. Ganha-se
    /// um relógio só no sistema, sem uma trait e um genérico atravessando três
    /// providers para entregar o que uma chamada de função entrega.
    fn call(&mut self, request: Request) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let request_id = request
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();

        let span = tracing::info_span!(
            SPAN,
            request_id = %request_id,
            method = %request.method(),
            path = %request.uri().path(),
        );

        let logger = self.logger.clone();
        let started = Utc::now();

        Box::pin(
            async move {
                let response = inner.call(request).await?;

                let elapsed = Utc::now().signed_duration_since(started).num_milliseconds();
                let status = response.status().as_u16().to_string();
                let duration = elapsed.to_string();
                let fields = [("status", status.as_str()), ("duration_ms", &duration)];

                if response.status().is_server_error() {
                    logger.error("requisição falhou", fields);
                } else {
                    logger.info("requisição atendida", fields);
                }

                Ok(response)
            }
            .instrument(span),
        )
    }
}
