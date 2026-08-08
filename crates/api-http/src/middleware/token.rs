//! O serviço de autenticação stateless.

use std::task::{Context, Poll};

use axum::extract::Request;
use axum::response::Response;
use futures::future::BoxFuture;
use tower::Service;

use crate::cookie::AuthCookie;
use crate::session::Session;
use crate::token::token_service::TokenService;

/// O serviço que resolve a sessão.
#[derive(Clone)]
pub struct Token<S> {
    /// O serviço interno, que este envolve.
    pub(super) inner: S,
    /// Quem emite e confere o access token.
    pub(super) tokens: TokenService,
    /// Como os cookies de sessão são escritos e lidos.
    pub(super) cookies: AuthCookie,
}

impl<S> Service<Request> for Token<S>
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

    fn call(&mut self, request: Request) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let presented = self.cookies.read_access(request.headers());
        let user = presented.and_then(|token| self.tokens.verify(&token));

        Box::pin(async move { Session::scope(user, inner.call(request)).await })
    }
}
