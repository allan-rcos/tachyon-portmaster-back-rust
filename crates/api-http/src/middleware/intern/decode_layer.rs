//! O middleware que negocia o formato do corpo que chegou.

use std::task::{Context, Poll};

use axum::extract::Request;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse as _, Response};
use futures::future::BoxFuture;
use tower::{Layer, Service};

use crate::middleware::intern::decode_context::DecodeContext;
use crate::ports::error::api_error::ApiError;
use crate::wire::media_type::MediaType;

/// Resolve o `Content-Type` e abre o escopo do formato de entrada.
///
/// ## O layer é o serviço antes de saber o que envolve
///
/// Um tipo só, e não um par. `DecodeLayer` sem parâmetro é o `Layer`, e
/// `DecodeLayer<S>` é o `Service` que sai do `layer()`.
///
/// Separado do [`EncodeLayer`](super::encode_layer::EncodeLayer) porque
/// `Content-Type` e `Accept` são cabeçalhos independentes: um cliente que manda
/// `FlatBuffers` e pede JSON de volta é um caso normal, e um layer só teria que
/// decidir os dois de uma vez ou carregar as duas variantes no mesmo escopo.
#[derive(Clone, Copy, Default)]
pub(crate) struct DecodeLayer<S = ()> {
    /// O serviço interno, que este envolve; `()` enquanto é só layer.
    inner: S,
}

impl DecodeLayer {
    /// Monta o layer.
    pub(crate) const fn new() -> Self {
        Self { inner: () }
    }
}

impl<S> Layer<S> for DecodeLayer {
    type Service = DecodeLayer<S>;

    fn layer(&self, inner: S) -> Self::Service {
        DecodeLayer { inner }
    }
}

impl<S> Service<Request> for DecodeLayer<S>
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

    /// Resolve o formato, ou recusa com `415`.
    ///
    /// `415 Unsupported Media Type` e não `406`: o `406` responde sobre o que
    /// **sai**, e aqui o problema é o que entrou. Recusar antes de ler o corpo é
    /// o que evita gastar o `Content-Length` de um cliente que anunciou um
    /// formato que não temos como interpretar.
    ///
    /// Cabeçalho ausente não é recusa: quem manda corpo sem anunciar o tipo é um
    /// cliente nosso falando o formato nativo, e é assim desde sempre.
    fn call(&mut self, request: Request) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let content_type = request
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok());

        let media = match MediaType::of_request(content_type) {
            Ok(media) => media,
            Err(refused) => {
                return Box::pin(async move {
                    Ok(
                        ApiError::new(StatusCode::UNSUPPORTED_MEDIA_TYPE, refused.to_string())
                            .into_response(),
                    )
                })
            }
        };

        Box::pin(async move { DecodeContext::scope(media, inner.call(request)).await })
    }
}
