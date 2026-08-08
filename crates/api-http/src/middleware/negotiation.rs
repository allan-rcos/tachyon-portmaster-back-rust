//! O serviço que anota cada requisição com o formato negociado.

use std::sync::Arc;
use std::task::{Context, Poll};

use axum::extract::Request;
use axum::http::header;
use axum::response::Response;
use futures::future::BoxFuture;
use tower::Service;

use crate::wire::media_type::MediaType;
use crate::wire::strategy::encode_strategy::EncodeStrategy;
use crate::wire::wire::Wire;

/// Lê `Content-Type` e `Accept` e guarda a decisão nas extensions.
#[derive(Clone)]
pub struct Negotiation<S> {
    inner: S,
    json: Arc<dyn EncodeStrategy>,
    flatbuffers: Arc<dyn EncodeStrategy>,
}

impl<S> Negotiation<S> {
    /// Monta o serviço sobre as strategies criadas no boot.
    pub(super) const fn new(
        inner: S,
        json: Arc<dyn EncodeStrategy>,
        flatbuffers: Arc<dyn EncodeStrategy>,
    ) -> Self {
        Self {
            inner,
            json,
            flatbuffers,
        }
    }
}

impl<S> Service<Request> for Negotiation<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        let (request_media, response_media) = {
            let headers = request.headers();
            let read =
                |name: &header::HeaderName| headers.get(name).and_then(|value| value.to_str().ok());

            (
                MediaType::of_request(read(&header::CONTENT_TYPE)),
                MediaType::of_response(read(&header::ACCEPT)),
            )
        };

        let encode = match response_media {
            MediaType::Json => self.json.clone(),
            MediaType::FlatBuffers => self.flatbuffers.clone(),
        };

        request
            .extensions_mut()
            .insert(Wire::new(request_media, encode));

        // `clone` antes do `call`: o `poll_ready` foi feito sobre `self.inner`,
        // e é esse serviço — não o clone — que está pronto.
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move { inner.call(request).await })
    }
}
