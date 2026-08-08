//! A resposta que não tem o que dizer.

use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

/// Uma resposta sem corpo, que ainda pode carregar cookies.
///
/// `204` é o que o PHP devolvia em refresh, logout e nas operações cujo
/// resultado é o próprio estado ter mudado.
///
/// Não passa pelo [`Wire`](crate::wire::wire::Wire) de propósito: sem corpo não
/// há o que negociar, e pedir a negociação aqui obrigaria toda rota de `204` a
/// declarar um extractor que ela não usa.
#[derive(Debug, Default)]
pub struct NoContent {
    cookies: Vec<String>,
}

impl NoContent {
    /// Uma resposta `204` vazia.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Acrescenta um `Set-Cookie`.
    #[must_use]
    pub(crate) fn with_cookie(mut self, cookie: String) -> Self {
        self.cookies.push(cookie);
        self
    }
}

impl IntoResponse for NoContent {
    fn into_response(self) -> Response {
        let mut response = StatusCode::NO_CONTENT.into_response();

        for cookie in self.cookies {
            if let Ok(value) = cookie.parse() {
                response.headers_mut().append(header::SET_COOKIE, value);
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
            .with_cookie("auth_token=; Max-Age=0".into())
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
            .with_cookie("auth_token=a; Path=/".into())
            .with_cookie("refresh_token=b; Path=/".into())
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
