//! O serviço de identificador de correlação.

use std::task::{Context, Poll};

use axum::extract::Request;
use axum::http::HeaderValue;
use axum::response::Response;
use futures::future::BoxFuture;
use portmaster_app::SequentialIdGenerator;
use tower::Service;

use super::request_id_header::REQUEST_ID_HEADER;

/// O serviço que carimba a requisição.
///
/// ## O id é sempre nosso
///
/// Um `X-Request-Id` que chegue de fora é **ignorado**. Antes ele era aceito se
/// tivesse até 128 bytes, o que deixava o cliente escolher o que apareceria em
/// toda linha de log daquela requisição — e num agregador de log, um valor
/// escolhido por quem faz a chamada é um valor que dá para forjar, colidir com o
/// de outra requisição ou usar para injetar conteúdo em quem lê os campos.
///
/// A resposta continua trazendo o id no cabeçalho, então quem chamou consegue
/// correlacionar. O que ele não consegue é **escolhê-lo** — nem quando a API
/// chama a si mesma, caso em que a chamada de dentro ganha um id próprio e a
/// cadeia se reconstrói pelo log, não pelo cabeçalho.
#[derive(Clone)]
pub(crate) struct RequestId<S, G> {
    /// O serviço interno, que este envolve.
    pub(super) inner: S,
    /// De onde sai o id desta requisição.
    pub(super) generator: G,
}

impl<S, G> Service<Request> for RequestId<S, G>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    G: SequentialIdGenerator + Clone + Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let id = self.generator.next();

        if let Ok(value) = HeaderValue::from_str(&id) {
            request.headers_mut().insert(REQUEST_ID_HEADER, value);
        }

        Box::pin(async move {
            let mut response = inner.call(request).await?;

            if let Ok(value) = HeaderValue::from_str(&id) {
                response.headers_mut().insert(REQUEST_ID_HEADER, value);
            }

            Ok(response)
        })
    }
}
