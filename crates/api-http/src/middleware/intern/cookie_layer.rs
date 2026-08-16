//! O middleware que abre o escopo dos cookies e os carimba na saída.

use std::sync::Arc;
use std::task::{Context, Poll};

use axum::extract::Request;
use axum::http::{header, HeaderValue};
use axum::response::Response;
use futures::future::BoxFuture;
use tower::{Layer, Service};

use crate::middleware::intern::cookie_context::CookieContext;

/// Abre o escopo dos cookies e recolhe o que o handler escreveu.
///
/// ## O layer é o serviço antes de saber o que envolve
///
/// Um tipo só, e não um par. `CookieLayer` sem parâmetro é o `Layer`, e
/// `CookieLayer<S>` é o `Service` que sai do `layer()`.
///
/// ## Ele fecha o próprio escopo, e só o dele
///
/// Ao contrário do escopo da `infra`, que reúne todos os contextos por tarefa
/// num mapa só e os confirma juntos, este abre e fecha sozinho. Não há motivo
/// para registrá-lo num escopo global: os contextos desta camada são
/// independentes entre si — nenhum depende de outro ter fechado antes — e cada
/// um tem exatamente um layer que o abre.
///
/// ## Onde ele fica na pilha
///
/// **Fora** do `SessionLayer`, que lê o access token pelo `CookiePort`, e fora
/// de todo handler que emite cookie.
#[derive(Clone, Copy, Default)]
pub(crate) struct CookieLayer<S = ()> {
    /// O serviço interno, que este envolve; `()` enquanto é só layer.
    inner: S,
}

impl CookieLayer {
    /// Monta o layer.
    pub(crate) const fn new() -> Self {
        Self { inner: () }
    }
}

impl<S> Layer<S> for CookieLayer {
    type Service = CookieLayer<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CookieLayer { inner }
    }
}

impl<S> Service<Request> for CookieLayer<S>
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

    /// Parseia o cabeçalho `Cookie`, roda o handler, e carimba o que ele
    /// escreveu.
    ///
    /// Um cabeçalho `Set-Cookie` por cookie: dois num só não são lidos por
    /// navegador nenhum.
    ///
    /// O carimbo acontece **depois** do handler, e por isso vale para toda
    /// resposta — inclusive a de erro. Deixá-lo a cargo de cada rota que emite
    /// cookie faria todo caminho de falha que precisa limpar a sessão lembrar de
    /// repetir o mesmo `fold`.
    fn call(&mut self, request: Request) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let jar = CookieContext::open(request.headers());
        let closing = Arc::clone(&jar);

        Box::pin(async move {
            let mut response = CookieContext::scope(jar, inner.call(request)).await?;

            let headers = response.headers_mut();
            for cookie in CookieContext::drain(&closing) {
                if let Ok(value) = HeaderValue::from_str(&cookie.to_string()) {
                    headers.append(header::SET_COOKIE, value);
                }
            }

            Ok(response)
        })
    }
}
