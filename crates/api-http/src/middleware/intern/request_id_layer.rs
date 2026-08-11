//! O middleware que carimba e publica o id de correlação.

use std::task::{Context, Poll};

use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue};
use axum::response::Response;
use futures::future::BoxFuture;
use portmaster_app::SequentialIdGenerator;
use tower::{Layer, Service};

use crate::middleware::intern::request_id_context::RequestIdContext;

/// O cabeçalho que devolve o id ao cliente.
const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// Carimba a requisição com um id de correlação e abre o escopo dele.
///
/// ## O layer é o serviço antes de saber o que envolve
///
/// Um tipo só, e não um par. `RequestIdLayer<G>` é o `Layer` — o gerador, ainda
/// sem serviço interno —, e `RequestIdLayer<G, S>` é o `Service` que sai do
/// `layer()`. Eram dois tipos em dois arquivos, e o segundo nunca foi nomeado
/// por ninguém.
///
/// ## O id é sempre nosso
///
/// Um `X-Request-Id` que chegue de fora é **ignorado**. Antes ele era aceito se
/// tivesse até 128 bytes, o que deixava o cliente escolher o que apareceria em
/// toda linha de log daquela requisição — e num agregador de log, um valor
/// escolhido por quem faz a chamada é um valor que dá para forjar, colidir com o
/// de outra requisição ou usar para injetar conteúdo em quem lê os campos.
///
/// A resposta continua trazendo o id no cabeçalho, então quem chamou consegue
/// correlacionar. O que ele não consegue é **escolhê-lo** — nem quando a API
/// chama a si mesma, caso em que a chamada de dentro ganha um id próprio e a
/// cadeia se reconstrói pelo log, não pelo cabeçalho.
///
/// ## O id não viaja na requisição
///
/// Ele viaja no escopo da tarefa. Antes era carimbado num cabeçalho da
/// requisição para o log encontrá-lo logo adiante — um dado nosso dando a volta
/// por uma estrutura do cliente, no mesmo campo em que o cliente podia ter
/// escrito. Quem quer o id agora pede ao
/// [`RequestIdPort`](crate::middleware::request_id_port::RequestIdPort).
#[derive(Clone)]
pub(crate) struct RequestIdLayer<G, S = ()> {
    /// O serviço interno, que este envolve; `()` enquanto é só layer.
    inner: S,
    /// De onde sai o id desta requisição.
    generator: G,
}

impl<G> RequestIdLayer<G> {
    /// Monta o layer com o gerador que o provider entregou.
    pub(crate) const fn new(generator: G) -> Self {
        Self {
            inner: (),
            generator,
        }
    }
}

impl<S, G: Clone> Layer<S> for RequestIdLayer<G> {
    type Service = RequestIdLayer<G, S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestIdLayer {
            inner,
            generator: self.generator.clone(),
        }
    }
}

impl<S, G> Service<Request> for RequestIdLayer<G, S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    G: SequentialIdGenerator + Clone + Send + 'static,
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

        let id = self.generator.next();
        let published = HeaderValue::from_str(&id).ok();

        Box::pin(async move {
            let mut response = RequestIdContext::scope(id, inner.call(request)).await?;

            if let Some(value) = published {
                response.headers_mut().insert(REQUEST_ID_HEADER, value);
            }

            Ok(response)
        })
    }
}
