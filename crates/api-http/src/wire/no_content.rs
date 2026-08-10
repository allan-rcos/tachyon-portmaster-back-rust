//! A resposta que não tem o que dizer.

use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use cookie::Cookie;

/// Uma resposta sem corpo, que ainda pode carregar cookies.
///
/// `204` é o que o PHP devolvia em refresh, logout e nas operações cujo
/// resultado é o próprio estado ter mudado.
///
/// Não passa pelo [`Encoder`](crate::wire::encoder::Encoder) de propósito: sem
/// corpo não há o que negociar, e pedir a negociação aqui obrigaria toda rota de
/// `204` a declarar um extractor que ela não usa.
#[derive(Debug, Default)]
pub(crate) struct NoContent {
    /// Os `Set-Cookie` a acrescentar na resposta, um cabeçalho por entrada.
    cookies: Vec<Cookie<'static>>,
}

impl NoContent {
    /// Uma resposta `204` vazia.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Acrescenta um `Set-Cookie`.
    #[must_use]
    pub(crate) fn with_cookie(mut self, cookie: Cookie<'static>) -> Self {
        self.cookies.push(cookie);
        self
    }
}

impl IntoResponse for NoContent {
    fn into_response(self) -> Response {
        let mut response = StatusCode::NO_CONTENT.into_response();
        let headers = response.headers_mut();

        for cookie in self.cookies {
            if let Ok(value) = HeaderValue::from_str(&cookie.to_string()) {
                headers.append(header::SET_COOKIE, value);
            }
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn a_resposta_vazia_ainda_carrega_cookies() {
        // É o caso do logout: nada a dizer, mas há cookies a limpar.
        let response = NoContent::new()
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

    #[test]
    fn dois_cookies_saem_em_cabecalhos_separados() {
        // Dois `Set-Cookie` num cabeçalho só não são lidos por navegador nenhum.
        let response = NoContent::new()
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
}
