//! O layer de autenticação stateless.

use tower::Layer;

use super::token::Token;
use crate::ports::cookie::auth_cookie::AuthCookie;
use crate::ports::token::token_service::TokenService;

/// Aplica o [`Token`].
#[derive(Clone)]
pub(crate) struct TokenLayer<T, A> {
    /// Quem confere o access token.
    tokens: T,
    /// De onde o access token é lido.
    cookies: A,
}

impl<T, A> TokenLayer<T, A> {
    /// Monta o layer com o que o provider entregou.
    pub(crate) const fn new(tokens: T, cookies: A) -> Self {
        Self { tokens, cookies }
    }
}

impl<S, T: TokenService, A: AuthCookie> Layer<S> for TokenLayer<T, A> {
    type Service = Token<S, T, A>;

    fn layer(&self, inner: S) -> Self::Service {
        Token {
            inner,
            tokens: self.tokens.clone(),
            cookies: self.cookies.clone(),
        }
    }
}
