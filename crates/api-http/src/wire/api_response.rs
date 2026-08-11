//! A resposta de um controller, já negociada.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use cookie::Cookie;

use crate::ports::error::api_error::ApiError;
use crate::wire::encoder::Encoder;
use crate::wire::x::response_x::ResponseX;

/// O que um controller produziu, pronto para virar resposta.
///
/// Genérica sobre o VO. Não há `Box<dyn>` aqui, e não há como haver: o corpo é
/// um tipo concreto que a rota conhece em tempo de compilação, e o
/// [`Encoder`] é monomorfizado junto com ele.
///
/// ## Por que ela carrega um `Result`
///
/// Porque o erro precisa sair pelo mesmo encoder que o acerto. Se o sucesso
/// passasse por aqui e a falha por outro caminho, existiriam dois lugares
/// escrevendo corpo — e um deles acabaria escrevendo num formato que o cliente
/// não pediu. Envolvendo o `Result`, negociar é uma coisa só que acontece uma
/// vez, no [`IntoResponse`] abaixo.
pub(crate) struct ApiResponse<X: ResponseX> {
    /// Como escrever o corpo.
    encoder: Encoder,
    /// O status do acerto — o da falha vem do próprio erro.
    status: StatusCode,
    /// O que responder, ou por que não dá.
    body: Result<X, ApiError>,
    /// Os `Set-Cookie` a acrescentar, um cabeçalho por entrada.
    cookies: Vec<Cookie<'static>>,
}

impl<X: ResponseX> ApiResponse<X> {
    /// Um `200` com o que o controller devolveu.
    pub(crate) const fn ok(encoder: Encoder, body: Result<X, ApiError>) -> Self {
        Self::with_status(encoder, StatusCode::OK, body)
    }

    /// Um `201` para o recurso recém-criado.
    pub(crate) const fn created(encoder: Encoder, body: Result<X, ApiError>) -> Self {
        Self::with_status(encoder, StatusCode::CREATED, body)
    }

    /// A resposta com um status escolhido.
    pub(crate) const fn with_status(
        encoder: Encoder,
        status: StatusCode,
        body: Result<X, ApiError>,
    ) -> Self {
        Self {
            encoder,
            status,
            body,
            cookies: Vec::new(),
        }
    }

    /// Acrescenta um `Set-Cookie` à resposta.
    #[must_use]
    pub(crate) fn with_cookie(mut self, cookie: Cookie<'static>) -> Self {
        self.cookies.push(cookie);
        self
    }
}

impl<X: ResponseX> IntoResponse for ApiResponse<X> {
    /// Codifica o corpo — ou o problema — e carimba os cookies.
    ///
    /// Os cookies do erro entram junto com os da resposta: uma recusa às vezes
    /// **precisa** mexer na sessão, e um refresh token morto tem que sair do
    /// navegador junto com o 401.
    fn into_response(self) -> Response {
        let mut cookies = self.cookies;

        match self.body {
            Ok(body) => self.encoder.respond(self.status, &body, cookies),
            Err(error) => {
                let (status, problem, from_error) = error.into_parts();
                cookies.extend(from_error);

                self.encoder.respond(status, &problem, cookies)
            }
        }
    }
}
