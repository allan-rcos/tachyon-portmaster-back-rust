//! O layer de autenticação stateless.

use tower::Layer;

use super::token::Token;
use crate::cookie::AuthCookie;
use crate::token::token_service::TokenService;

/// Aplica o [`Token`].
#[derive(Clone)]
pub struct TokenLayer {
    /// Quem emite e confere o access token.
    tokens: TokenService,
    /// Como os cookies de sessão são escritos e lidos.
    cookies: AuthCookie,
}

impl TokenLayer {
    /// Monta o layer com o serviço de token e os cookies.
    pub(crate) const fn new(tokens: TokenService, cookies: AuthCookie) -> Self {
        Self { tokens, cookies }
    }
}

impl<S> Layer<S> for TokenLayer {
    type Service = Token<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Token {
            inner,
            tokens: self.tokens.clone(),
            cookies: self.cookies.clone(),
        }
    }
}
