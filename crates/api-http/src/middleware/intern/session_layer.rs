//! O middleware que resolve a sessão a partir do token apresentado.

use std::task::{Context, Poll};

use axum::extract::Request;
use axum::response::Response;
use futures::future::BoxFuture;
use tower::{Layer, Service};

use crate::middleware::cookie_port::CookiePort as _;
use crate::middleware::intern::cookie_context::CookieContext;
use crate::middleware::intern::session_context::SessionContext;
use crate::ports::cookie::cookie_name::CookieName;
use crate::ports::token::token_service::TokenService;

/// Resolve a sessão a partir do token apresentado e abre o escopo dela.
///
/// Genérico sobre o serviço de token: ele pede "o principal deste token" sem
/// conhecer a impl. O token em si vem pelo `CookiePort`, o que significa que
/// este layer tem de ficar **dentro** do `CookieLayer`.
///
/// ## O layer é o serviço antes de saber o que envolve
///
/// Um tipo só, e não um par. `SessionLayer<T>` é o `Layer`, e
/// `SessionLayer<T, S>` é o `Service` que sai do `layer()`.
#[derive(Clone)]
pub(crate) struct SessionLayer<T, S = ()> {
    /// O serviço interno, que este envolve; `()` enquanto é só layer.
    inner: S,
    /// Quem confere o access token.
    tokens: T,
}

impl<T> SessionLayer<T> {
    /// Monta o layer com o que o provider entregou.
    pub(crate) const fn new(tokens: T) -> Self {
        Self { inner: (), tokens }
    }
}

impl<S, T: TokenService> Layer<S> for SessionLayer<T> {
    type Service = SessionLayer<T, S>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionLayer {
            inner,
            tokens: self.tokens.clone(),
        }
    }
}

impl<S, T> Service<Request> for SessionLayer<T, S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    T: TokenService,
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
    /// [`SessionPort::require_user`](crate::middleware::session_port::SessionPort::require_user()).
    fn call(&mut self, request: Request) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let presented = CookieContext.read(CookieName::Access).ok().flatten();
        let user = presented.and_then(|token| self.tokens.verify(&token));

        Box::pin(async move { SessionContext::scope(user, inner.call(request)).await })
    }
}
