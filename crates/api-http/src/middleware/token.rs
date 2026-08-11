//! O serviço de autenticação stateless.

use std::task::{Context, Poll};

use axum::extract::Request;
use axum::response::Response;
use futures::future::BoxFuture;
use tower::Service;

use crate::ports::cookie::auth_cookie::AuthCookie;
use crate::session::Session;
use crate::ports::token::token_service::TokenService;

/// O serviço que resolve a sessão.
///
/// Genérico sobre o token e sobre os cookies: ele pede "o token apresentado" e
/// "o principal deste token" sem conhecer nenhuma das duas impls.
#[derive(Clone)]
pub(crate) struct Token<S, T, A> {
    /// O serviço interno, que este envolve.
    pub(super) inner: S,
    /// Quem confere o access token.
    pub(super) tokens: T,
    /// De onde o access token é lido.
    pub(super) cookies: A,
}

impl<S, T, A> Service<Request> for Token<S, T, A>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    T: TokenService,
    A: AuthCookie,
{
    type Response = Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    /// Abre o escopo de sessão da requisição.
    ///
    /// A conferência é síncrona e acontece **antes** do `Box::pin`: ela não toca
    /// o banco — tudo que a autorização precisa saber veio assinado dentro do
    /// token — e por isso não há o que esperar.
    ///
    /// Sem token, ou com um que não vale, o escopo abre com `None`. Recusar aqui
    /// fecharia as rotas públicas junto; quem exige sessão é o controller, com
    /// [`Session::require_user`].
    fn call(&mut self, request: Request) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let presented = self.cookies.read_access(request.headers());
        let user = presented.and_then(|token| self.tokens.verify(&token));

        Box::pin(async move { Session::scope(user, inner.call(request)).await })
    }
}
