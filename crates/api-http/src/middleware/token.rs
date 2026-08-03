//! Autenticação stateless: valida o token e abre o escopo de sessão.
//!
//! **Nunca toca o banco.** É a regra de ouro da autenticação aqui: este código
//! roda a cada requisição, e uma consulta por requisição só para descobrir quem
//! está falando é justamente o que a sessão auto-contida evita. Tudo que a
//! autorização precisa já viaja assinado dentro do token.
//!
//! ## Não rejeita nada
//!
//! Um token ausente ou inválido não vira 401 aqui — vira **ausência de sessão**.
//! Quem decide se a rota exige login é o handler, porque só ele sabe: `/info` e
//! `/auth/login` são públicas, e recusar no middleware as tornaria inalcançáveis.
//!
//! O que este middleware garante é que, ao chegar no handler, a pergunta "há
//! sessão?" já tem resposta — e que ela foi obtida conferindo a assinatura.

use std::task::{Context, Poll};

use axum::extract::Request;
use axum::response::Response;
use futures::future::BoxFuture;
use tower::{Layer, Service};

use crate::cookie::AuthCookie;
use crate::session::Session;
use crate::token::TokenService;

/// Aplica o [`Token`].
#[derive(Clone)]
pub(crate) struct TokenLayer {
    tokens: TokenService,
    cookies: AuthCookie,
}

impl TokenLayer {
    /// Monta o layer com o serviço de token e os cookies.
    pub(crate) fn new(tokens: TokenService, cookies: AuthCookie) -> Self {
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

/// O serviço que resolve a sessão.
#[derive(Clone)]
pub(crate) struct Token<S> {
    inner: S,
    tokens: TokenService,
    cookies: AuthCookie,
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

        // O token vem do cookie, e não de um `Authorization: Bearer`: é o que o
        // PHP fazia, e é o que permite ao cookie ser `HttpOnly` — um token que
        // o JavaScript precisa ler para pôr num cabeçalho é um token que um XSS
        // também lê.
        let presented = self.cookies.read_access(request.headers());
        let user = presented.and_then(|token| self.tokens.verify(&token));

        Box::pin(async move { Session::scope(user, inner.call(request)).await })
    }
}
