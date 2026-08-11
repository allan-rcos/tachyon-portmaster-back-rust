//! O middleware que negocia o formato da resposta.

use std::task::{Context, Poll};

use axum::extract::Request;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse as _, Response};
use futures::future::BoxFuture;
use tower::{Layer, Service};

use crate::middleware::intern::encode_context::EncodeContext;
use crate::ports::error::api_error::ApiError;
use crate::wire::media_type::MediaType;

/// Resolve o `Accept` e abre o escopo do formato de saída.
///
/// ## O layer é o serviço antes de saber o que envolve
///
/// Um tipo só, e não um par. `EncodeLayer` sem parâmetro é o `Layer`, e
/// `EncodeLayer<S>` é o `Service` que sai do `layer()`.
///
/// ## Onde ele fica na pilha
///
/// **Fora** do `Recover` e do `Timeout`. Os dois produzem resposta por conta
/// própria — um `500` e um `504` — e precisam do formato já resolvido para
/// escrevê-la; antes cada um relia o `Accept` da requisição por conta própria,
/// que é a mesma decisão tomada em três lugares.
#[derive(Clone, Copy, Default)]
pub(crate) struct EncodeLayer<S = ()> {
    /// O serviço interno, que este envolve; `()` enquanto é só layer.
    inner: S,
}

impl EncodeLayer {
    /// Monta o layer.
    pub(crate) const fn new() -> Self {
        Self { inner: () }
    }
}

impl<S> Layer<S> for EncodeLayer {
    type Service = EncodeLayer<S>;

    fn layer(&self, inner: S) -> Self::Service {
        EncodeLayer { inner }
    }
}

impl<S> Service<Request> for EncodeLayer<S>
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

    /// Resolve o formato, ou recusa com `406`.
    ///
    /// `406 Not Acceptable` é literalmente o que aconteceu: o cliente listou o
    /// que aceita e não há interseção com o que sabemos escrever. Antes isto
    /// caía em JSON calado, e quem pedira XML recebia um `200` com um corpo que
    /// não sabe ler — nem recebe o que pediu, nem descobre que não vai receber.
    ///
    /// O corpo da própria recusa sai em JSON, que é o padrão do
    /// [`EncodeContext`] fora do escopo. Não há alternativa: acabamos de
    /// declarar que não sabemos escrever no que ele pediu.
    fn call(&mut self, request: Request) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let accept = request
            .headers()
            .get(header::ACCEPT)
            .and_then(|value| value.to_str().ok());

        let media = match MediaType::of_response(accept) {
            Ok(media) => media,
            Err(refused) => {
                return Box::pin(async move {
                    Ok(
                        ApiError::new(StatusCode::NOT_ACCEPTABLE, refused.to_string())
                            .into_response(),
                    )
                })
            }
        };

        Box::pin(async move { EncodeContext::scope(media, inner.call(request)).await })
    }
}
