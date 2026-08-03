//! O registro de cada requisição.
//!
//! Uma linha por requisição, com método, rota, status e duração — e o
//! `request_id` que o [`super::request_id`] carimbou, para que as linhas de uma
//! mesma requisição se juntem.
//!
//! ## O que não é logado
//!
//! Corpo, cabeçalhos e querystring. O corpo carrega senha em `/auth/login` e em
//! todo endpoint de troca de senha; os cabeçalhos carregam o cookie de sessão; e
//! a querystring carrega termos de busca que são dado do cliente. Logar
//! "method path status duration" é o que responde às perguntas operacionais sem
//! transformar o arquivo de log num vazamento em repouso.

use std::task::{Context, Poll};
use std::time::Instant;

use axum::extract::Request;
use axum::response::Response;
use futures::future::BoxFuture;
use portmaster_app::{Logger, LoggerFactory};
use tower::{Layer, Service};

use super::request_id::REQUEST_ID_HEADER;

/// O nome do componente nos logs.
const CHANNEL: &str = "http";

/// Aplica o [`Logging`].
#[derive(Clone)]
pub(crate) struct LoggingLayer<F> {
    factory: F,
}

impl<F> LoggingLayer<F> {
    /// Monta o layer com a fábrica que o provider entregou.
    pub(crate) fn new(factory: F) -> Self {
        Self { factory }
    }
}

impl<S, F: LoggerFactory> Layer<S> for LoggingLayer<F> {
    type Service = Logging<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Logging {
            inner,
            logger: self.factory.create(CHANNEL),
        }
    }
}

/// O serviço que registra a requisição.
#[derive(Clone)]
pub(crate) struct Logging<S> {
    inner: S,
    logger: Logger,
}

impl<S> Service<Request> for Logging<S>
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

        let method = request.method().to_string();
        let path = request.uri().path().to_owned();
        let request_id = request
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();

        let logger = self
            .logger
            .with_field("request_id", request_id)
            .with_field("method", method)
            .with_field("path", path);

        Box::pin(async move {
            let started = Instant::now();
            let response = inner.call(request).await?;

            let logger = logger
                .with_field("status", response.status().as_u16().to_string())
                .with_field("duration_ms", started.elapsed().as_millis().to_string());

            // O nível segue o status: um 500 precisa aparecer numa busca por
            // erro, e um 404 de rota inexistente não é incidente de ninguém.
            if response.status().is_server_error() {
                logger.error("requisição falhou");
            } else {
                logger.info("requisição atendida");
            }

            Ok(response)
        })
    }
}
