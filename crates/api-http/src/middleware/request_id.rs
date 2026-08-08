//! O serviço de identificador de correlação.

use std::task::{Context, Poll};

use axum::extract::Request;
use axum::http::HeaderValue;
use axum::response::Response;
use futures::future::BoxFuture;
use portmaster_app::SortableIdGenerator;
use tower::Service;

use super::request_id_header::REQUEST_ID_HEADER;

/// O serviço que carimba a requisição.
#[derive(Clone)]
pub struct RequestId<S, G> {
    pub(super) inner: S,
    pub(super) generator: G,
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
