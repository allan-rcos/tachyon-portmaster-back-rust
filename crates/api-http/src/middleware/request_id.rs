//! O identificador de correlação de uma requisição.
//!
//! Um id **ordenável** (xid), não aleatório: os logs de uma requisição precisam
//! se sequenciar, e um id que ordena por emissão deixa isso de graça ao ordenar
//! por texto. Não é chave primária de nada — nasce aqui e morre no log.
//!
//! O id volta no cabeçalho `X-Request-Id` para que quem abre um chamado possa
//! citar exatamente a requisição que falhou, em vez de descrever a hora
//! aproximada.

use std::task::{Context, Poll};

use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue};
use axum::response::Response;
use futures::future::BoxFuture;
use portmaster_app::SortableIdGenerator;
use tower::{Layer, Service};

/// O cabeçalho que devolve o id ao cliente.
pub(crate) const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// Aplica o [`RequestId`].
#[derive(Clone)]
pub(crate) struct RequestIdLayer<G> {
    generator: G,
}

impl<G> RequestIdLayer<G> {
    /// Monta o layer com o gerador que o provider entregou.
    pub(crate) fn new(generator: G) -> Self {
        Self { generator }
    }
}

impl<S, G: Clone> Layer<S> for RequestIdLayer<G> {
    type Service = RequestId<S, G>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestId {
            inner,
            generator: self.generator.clone(),
        }
    }
}

/// O serviço que carimba a requisição.
#[derive(Clone)]
pub(crate) struct RequestId<S, G> {
    inner: S,
    generator: G,
}

impl<S, G> Service<Request> for RequestId<S, G>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    G: SortableIdGenerator + Clone + Send + 'static,
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

        // Um id que o cliente mandou é aceito: é o que permite correlacionar uma
        // cadeia de serviços. Só é gerado um novo quando não veio nenhum — ou
        // quando o que veio não cabe num cabeçalho.
        let id = request
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|value| value.to_str().ok())
            .filter(|value| !value.is_empty() && value.len() <= 128)
            .map_or_else(|| self.generator.next(), str::to_owned);

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
