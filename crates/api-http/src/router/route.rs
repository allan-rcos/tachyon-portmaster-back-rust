//! Uma linha de uma tabela de rotas.

use axum::handler::Handler;
use axum::http::Method;
use axum::routing::MethodRouter;

/// Um verbo, um caminho, e o handler que o par resolve.
///
/// **A identidade é o método e o caminho, não o handler.** É exatamente o par
/// que o axum recusa ver duas vezes — registrar o mesmo verbo no mesmo caminho
/// derruba o boot com "Overlapping method route" —, então duas linhas que
/// concordam nele são a mesma rota por mais diferentes que sejam os handlers.
///
/// É essa identidade que responde sozinha à pergunta do alias sem versão: o
/// [`RouterHub`](super::router_hub::RouterHub) percorre as versões da mais nova
/// para a mais velha e fica com a primeira ocorrência de cada par, o que deixa
/// cada rota apontando para a versão mais nova que ainda a publica.
///
/// Por que uma linha por verbo, e não um `MethodRouter` com todos: porque o
/// verbo faz parte da identidade. Um `GET /products` carregado da v1 para a v2 e
/// um `DELETE /products` que só existiu na v1 têm que poder resolver para
/// versões diferentes, e um par agrupado decidiria pelos dois de uma vez.
pub(crate) struct Route {
    /// O verbo HTTP.
    pub(crate) method: Method,
    /// O caminho, relativo ao prefixo de versão.
    pub(crate) path: &'static str,
    /// O que atende o par.
    pub(crate) handler: MethodRouter,
}

impl Route {
    /// Uma rota de leitura.
    pub(crate) fn get<H: Handler<T, ()>, T: 'static>(path: &'static str, handler: H) -> Self {
        Self {
            method: Method::GET,
            path,
            handler: axum::routing::get(handler),
        }
    }

    /// Uma rota de criação ou de ação.
    pub(crate) fn post<H: Handler<T, ()>, T: 'static>(path: &'static str, handler: H) -> Self {
        Self {
            method: Method::POST,
            path,
            handler: axum::routing::post(handler),
        }
    }

    /// Uma rota de substituição.
    pub(crate) fn put<H: Handler<T, ()>, T: 'static>(path: &'static str, handler: H) -> Self {
        Self {
            method: Method::PUT,
            path,
            handler: axum::routing::put(handler),
        }
    }

    /// Uma rota de remoção.
    pub(crate) fn delete<H: Handler<T, ()>, T: 'static>(path: &'static str, handler: H) -> Self {
        Self {
            method: Method::DELETE,
            path,
            handler: axum::routing::delete(handler),
        }
    }
}
