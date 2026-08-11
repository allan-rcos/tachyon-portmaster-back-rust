//! O middleware que resolve a sessão a partir do token.

use std::task::{Context, Poll};

use axum::extract::Request;
use axum::response::Response;
use futures::future::BoxFuture;
use tower::{Layer, Service};

use crate::ports::cookie::auth_cookie::AuthCookie;
use crate::ports::token::token_service::TokenService;
use crate::session::Session;

/// Resolve a sessão a partir do token apresentado e abre o escopo dela.
///
/// Genérico sobre o token e sobre os cookies: ele pede "o token apresentado" e
/// "o principal deste token" sem conhecer nenhuma das duas impls.
///
/// ## O layer é o serviço antes de saber o que envolve
///
/// Um tipo só, e não um par. `SessionLayer<T, A>` é o `Layer`, e
/// `SessionLayer<T, A, S>` é o `Service` que sai do `layer()`. Eram dois tipos
/// em dois arquivos, e o segundo nunca foi nomeado por ninguém.
#[derive(Clone)]
pub(crate) struct SessionLayer<T, A, S = ()> {
    /// O serviço interno, que este envolve; `()` enquanto é só layer.
    inner: S,
    /// Quem confere o access token.
    tokens: T,
    /// De onde o access token é lido.
    cookies: A,
}

impl<T, A> SessionLayer<T, A> {
    /// Monta o layer com o que o provider entregou.
    pub(crate) const fn new(tokens: T, cookies: A) -> Self {
        Self {
            inner: (),
            tokens,
            cookies,
        }
    }
}

impl<S, T: TokenService, A: AuthCookie> Layer<S> for SessionLayer<T, A> {
    type Service = SessionLayer<T, A, S>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionLayer {
            inner,
            tokens: self.tokens.clone(),
            cookies: self.cookies.clone(),
        }
    }
}

impl<S, T, A> Service<Request> for SessionLayer<T, A, S>
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
