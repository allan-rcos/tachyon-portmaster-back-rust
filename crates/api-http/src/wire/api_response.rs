//! A resposta de um controller, já negociada.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use cookie::Cookie;

use crate::middleware::encode_port::EncodePort as _;
use crate::middleware::intern::encode_context::EncodeContext;
use crate::ports::error::api_error::ApiError;
use crate::wire::vo::common::problem_x::ProblemX;
use crate::wire::x::response_x::ResponseX;

/// O que um controller produziu, pronto para virar resposta.
///
/// Genérica sobre o VO. Não há `Box<dyn>` aqui, e não há como haver: o corpo é
/// um tipo concreto que a rota conhece em tempo de compilação, e a codificação é
/// monomorfizada junto com ele.
///
/// ## Por que ela carrega um `Result`
///
/// Porque o erro precisa sair pelo mesmo formato que o acerto. Se o sucesso
/// passasse por aqui e a falha por outro caminho, existiriam dois lugares
/// escrevendo corpo — e um deles acabaria escrevendo num formato que o cliente
/// não pediu. Envolvendo o `Result`, negociar é uma coisa só que acontece uma
/// vez, no [`IntoResponse`] abaixo.
///
/// ## O parâmetro tem padrão
///
/// Porque uma resposta sem corpo não tem VO nenhum, e `ApiResponse::no_content()`
/// precisa resolver sozinho. `ProblemX` é o que ela usaria de qualquer forma se
/// o `Result` desse errado — o corpo de erro é o mesmo em toda resposta.
///
/// ## E por que o corpo é `Option`
///
/// Porque `204` é uma resposta como as outras. Era um tipo à parte — o
/// `NoContent` —, o que dava duas formas de responder e duas de anexar cookie a
/// uma resposta. Um controller que às vezes tem corpo e às vezes não tinha de
/// escolher entre elas no meio do método.
pub(crate) struct ApiResponse<X: ResponseX = ProblemX> {
    /// O status do acerto — o da falha vem do próprio erro.
    status: StatusCode,
    /// O que responder, ou por que não dá; `Ok(None)` é o `204`.
    body: Result<Option<X>, ApiError>,
    /// Os `Set-Cookie` a acrescentar, um cabeçalho por entrada.
    cookies: Vec<Cookie<'static>>,
}

impl<X: ResponseX> ApiResponse<X> {
    /// Um `200` com o que o controller devolveu.
    pub(crate) fn ok(body: Result<X, ApiError>) -> Self {
        Self::with_status(StatusCode::OK, body)
    }

    /// Um `201` para o recurso recém-criado.
    pub(crate) fn created(body: Result<X, ApiError>) -> Self {
        Self::with_status(StatusCode::CREATED, body)
    }

    /// A resposta com um status escolhido.
    pub(crate) fn with_status(status: StatusCode, body: Result<X, ApiError>) -> Self {
        Self {
            status,
            body: body.map(Some),
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

impl ApiResponse<ProblemX> {
    /// Um `204` para a operação cujo resultado é o próprio estado ter mudado.
    ///
    /// É o que o PHP devolvia em refresh, logout e afins. Sem corpo não há o que
    /// negociar, mas ainda pode haver cookie a carimbar — e é por isso que ela
    /// mora aqui e não num tipo separado.
    ///
    /// Fixada em `ProblemX` porque uma resposta sem corpo não tem VO nenhum, e o
    /// parâmetro precisa de algum: o `ProblemX` é o que ela usaria de qualquer
    /// forma se o `Result` desse errado. Fixá-la aqui é também o que faz
    /// `ApiResponse::no_content()` resolver sem turbofish.
    pub(crate) const fn no_content() -> Self {
        Self {
            status: StatusCode::NO_CONTENT,
            body: Ok(None),
            cookies: Vec::new(),
        }
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
            Ok(Some(body)) => EncodeContext.respond(self.status, &body, cookies),

            Ok(None) => {
                let mut response = self.status.into_response();
                append_cookies(&mut response, cookies);

                response
            }

            Err(error) => {
                let (status, problem, from_error) = error.into_parts();
                cookies.extend(from_error);

                EncodeContext.respond(status, &problem, cookies)
            }
        }
    }
}

/// Carimba os `Set-Cookie` numa resposta que não passou pelo encoder.
///
/// Um cabeçalho por cookie: dois `Set-Cookie` num só não são lidos por navegador
/// nenhum.
fn append_cookies(response: &mut Response, cookies: Vec<Cookie<'static>>) {
    let headers = response.headers_mut();

    for cookie in cookies {
        if let Ok(value) = axum::http::HeaderValue::from_str(&cookie.to_string()) {
            headers.append(axum::http::header::SET_COOKIE, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::header;
    use pretty_assertions::assert_eq;

    fn body() -> ProblemX {
        ProblemX {
            kind: "about:blank",
            title: "OK".to_owned(),
            status: 200,
            detail: "tudo certo".to_owned(),
        }
    }

    #[tokio::test]
    async fn a_resposta_vazia_ainda_carrega_cookies() {
        // É o caso do logout: nada a dizer, mas há cookies a limpar.
        let response = ApiResponse::no_content()
            .with_cookie(
                Cookie::build(("auth_token", ""))
                    .max_age(cookie::time::Duration::ZERO)
                    .build(),
            )
            .into_response();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            response
                .headers()
                .get_all(header::SET_COOKIE)
                .iter()
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn dois_cookies_saem_em_cabecalhos_separados() {
        let response = ApiResponse::no_content()
            .with_cookie(Cookie::build(("auth_token", "a")).path("/").build())
            .with_cookie(Cookie::build(("refresh_token", "b")).path("/").build())
            .into_response();

        assert_eq!(
            response
                .headers()
                .get_all(header::SET_COOKIE)
                .iter()
                .count(),
            2
        );
    }

    #[tokio::test]
    async fn o_corpo_de_acerto_sai_codificado() {
        let response = ApiResponse::ok(Ok(body())).into_response();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key(header::CONTENT_TYPE));
    }

    /// O erro sai pelo mesmo caminho do acerto, e com o status dele.
    #[tokio::test]
    async fn o_erro_carrega_o_proprio_status() {
        let response =
            ApiResponse::<ProblemX>::ok(Err(ApiError::unauthenticated())).into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
