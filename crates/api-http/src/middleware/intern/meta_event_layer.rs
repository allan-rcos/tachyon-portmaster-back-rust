//! O middleware que abre o escopo da pilha de eventos.

use std::task::{Context, Poll};

use axum::extract::Request;
use axum::response::Response;
use futures::future::BoxFuture;
use portmaster_app::MetaEventStackSubscriber;
use tower::{Layer, Service};

/// Abre o escopo em que a pilha de eventos da requisição existe, e nada além.
///
/// ## Não faz nada com o que foi registrado
///
/// Ele **abre**; quem lê é outro layer, e quem escreve é um caso de uso. Somar
/// as três coisas num middleware só faria este arquivo conhecer o
/// [`MetaEvent`](portmaster_app::MetaEvent) que ele não tem por que conhecer, e
/// um evento novo passaria a tocá-lo.
///
/// ## Quem lê tem de estar dentro
///
/// A leitura acontece depois que o handler responde, e um `task_local` só existe
/// dentro do futuro que o escopo embrulha. Então todo layer que pergunte à pilha
/// precisa ser **mais interno** que este — em `router/mod.rs`, onde o último
/// `.layer` é o mais externo, isso quer dizer aplicado antes dele.
///
/// Errar essa ordem não quebra nada de forma visível: a pergunta responde
/// `false` e a resposta sai sem a marca. É o teste de integração que pega, ao
/// afirmar o acerto e não só a presença do cabeçalho.
#[derive(Clone)]
pub(crate) struct MetaEventLayer<E, S = ()> {
    /// O serviço interno, que este envolve; `()` enquanto é só layer.
    inner: S,
    /// A pilha cujo escopo é aberto.
    events: E,
}

impl<E> MetaEventLayer<E> {
    /// Monta o layer com a pilha que o provider entregou.
    pub(crate) const fn new(events: E) -> Self {
        Self { inner: (), events }
    }
}

impl<S, E: Clone> Layer<S> for MetaEventLayer<E> {
    type Service = MetaEventLayer<E, S>;

    fn layer(&self, inner: S) -> Self::Service {
        MetaEventLayer {
            inner,
            events: self.events.clone(),
        }
    }
}

impl<S, E> Service<Request> for MetaEventLayer<E, S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    E: MetaEventStackSubscriber + Clone + Send + Sync + 'static,
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
        let events = self.events.clone();

        Box::pin(async move { events.scope(inner.call(request)).await })
    }
}
