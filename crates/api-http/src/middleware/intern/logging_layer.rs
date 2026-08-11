//! O middleware que registra a requisição.

use std::task::{Context, Poll};

use axum::extract::Request;
use axum::response::Response;
use chrono::Utc;
use futures::future::BoxFuture;
use portmaster_app::{Logger, LoggerFactory};
use tower::{Layer, Service};
use tracing::Instrument as _;

use crate::middleware::intern::request_id_context::RequestIdContext;
use crate::middleware::request_id_port::RequestIdPort as _;

/// O nome do componente nos logs.
const CHANNEL: &str = "http";

/// O nome do span que envolve a requisição inteira.
const SPAN: &str = "request";

/// Abre o span da requisição e registra o desfecho dela.
///
/// ## O layer é o serviço antes de saber o que envolve
///
/// Um tipo só, e não um par. `LoggingLayer<L>` é o `Layer` — o logger do canal,
/// ainda sem serviço interno —, e `LoggingLayer<L, S>` é o `Service` que sai do
/// `layer()`. Eram dois tipos em dois arquivos, e o segundo nunca foi nomeado
/// por ninguém.
///
/// O logger é criado **uma vez**, na construção do layer, e clonado por serviço.
/// Antes o layer guardava a fábrica e criava um logger a cada `layer()`, que é
/// trabalho repetido para produzir sempre o mesmo canal.
#[derive(Clone)]
pub(crate) struct LoggingLayer<L, S = ()> {
    /// O serviço interno, que este envolve; `()` enquanto é só layer.
    inner: S,
    /// O logger do canal HTTP, sem nada de uma requisição em particular.
    logger: L,
}

impl<L> LoggingLayer<L> {
    /// Monta o layer com a fábrica que o provider entregou.
    pub(crate) fn new<F: LoggerFactory<Instance = L>>(factory: &F) -> Self {
        Self {
            inner: (),
            logger: factory.create(CHANNEL),
        }
    }
}

impl<S, L: Clone> Layer<S> for LoggingLayer<L> {
    type Service = LoggingLayer<L, S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingLayer {
            inner,
            logger: self.logger.clone(),
        }
    }
}

impl<S, L> Service<Request> for LoggingLayer<L, S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    L: Logger,
{
    type Response = Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    /// Abre o span da requisição, mede a latência e emite a linha do desfecho.
    ///
    /// O span é o que faz o `request_id` alcançar quem não o recebeu: o resto da
    /// pilha corre dentro dele, então uma linha emitida lá no fundo do `app` ou
    /// da `infra` sai correlacionada sem que ninguém no meio do caminho tenha
    /// tocado no assunto. Carimbar o id num logger só alcançava quem tivesse
    /// aquele logger em mãos, e ninguém abaixo do middleware tinha.
    ///
    /// O id vem do escopo, e não de um cabeçalho da requisição. Este layer é um
    /// leitor do contexto como qualquer outro: pede ao
    /// [`RequestIdPort`](crate::middleware::request_id_port::RequestIdPort), que
    /// é a mesma porta que um controller usaria — e por isso precisa estar
    /// **dentro** do layer que abre aquele escopo.
    ///
    /// A latência sai de dois `Utc::now()`, e não de um `Instant::now` — que o
    /// `.clippy.toml` proíbe. Perde-se a monotonicidade: um ajuste de NTP no
    /// meio de uma resposta produziria um `duration_ms` esquisito, e isso é um
    /// evento que não acontece dentro de uma resposta de milissegundos. Ganha-se
    /// um relógio só no sistema, sem uma trait e um genérico atravessando três
    /// providers para entregar o que uma chamada de função entrega.
    fn call(&mut self, request: Request) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let span = tracing::info_span!(
            SPAN,
            request_id = %RequestIdContext.current().unwrap_or_default(),
            method = %request.method(),
            path = %request.uri().path(),
        );

        let logger = self.logger.clone();
        let started = Utc::now();

        Box::pin(
            async move {
                let response = inner.call(request).await?;

                let elapsed = Utc::now().signed_duration_since(started).num_milliseconds();
                let status = response.status().as_u16().to_string();
                let duration = elapsed.to_string();
                let fields = [("status", status.as_str()), ("duration_ms", &duration)];

                if response.status().is_server_error() {
                    logger.error("requisição falhou", fields);
                } else {
                    logger.info("requisição atendida", fields);
                }

                Ok(response)
            }
            .instrument(span),
        )
    }
}
