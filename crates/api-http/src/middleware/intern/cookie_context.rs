//! Os cookies da tarefa corrente: os que chegaram e os que vão sair.
//!
//! É o **único** arquivo do sistema que conhece o tipo `Cookie`. Havia uma trait
//! `AuthCookie` tentando abstraí-lo, e ela não abstraía nada: quatro dos seis
//! métodos devolviam um `Cookie` na assinatura, então o tipo interno atravessava
//! o contrato e chegava aos controllers de qualquer forma.
//!
//! > **Lacuna conhecida do `lint-exports`:** o `CURRENT` abaixo nasce de
//! > `tokio::task_local!`, e itens gerados por macro são invisíveis ao `syn`.
//! > O arquivo exporta dois itens na prática, não um.

use std::future::Future;
use std::sync::Arc;

use axum::http::{header, HeaderMap};
use cookie::Cookie;
use portmaster_app::{Logger as _, SystemLogger};

use crate::middleware::cookie_port::CookiePort;
use crate::ports::cookie::cookie_name::CookieName;
use crate::ports::error::api_error::ApiError;
use crate::ports::session_policy::SessionPolicy;

tokio::task_local! {
    /// Os cookies desta requisição.
    static CURRENT: Arc<CookieJar>;
}

/// O que chegou no cabeçalho `Cookie` e o que sairá em `Set-Cookie`.
///
/// O `Arc` existe porque o layer precisa recolher os pendentes **depois** de o
/// handler terminar, e um task-local é consumido ao abrir o escopo: ele guarda a
/// própria referência antes de entrar.
pub(super) struct CookieJar {
    /// O que o cliente apresentou, já parseado.
    incoming: Vec<(String, String)>,

    /// O que ainda vai sair na resposta.
    ///
    /// Não há disputa aqui: o escopo é de uma tarefa e as chamadas do handler
    /// são sequenciais. O `Mutex` existe para dar `Sync` ao `Arc` que o layer
    /// segura do lado de fora — sem ele, o `Arc` não atravessa o `.await` do
    /// handler.
    #[allow(
        clippy::disallowed_types,
        reason = "o .clippy.toml admite Mutex em recurso de borda justificado; é o Sync do handle que o layer segura fora do escopo, não estado compartilhado entre tarefas"
    )]
    pending: std::sync::Mutex<Vec<Cookie<'static>>>,
}

/// O adaptador que serve os cookies do escopo corrente.
///
/// ZST: os cookies são da tarefa, não do objeto.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct CookieContext;

impl CookieContext {
    /// Lê o cabeçalho `Cookie` e monta o jar da requisição.
    ///
    /// `pub(super)`: só o layer irmão abre um escopo.
    pub(super) fn open(headers: &HeaderMap) -> Arc<CookieJar> {
        let incoming = headers
            .get_all(header::COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .flat_map(Cookie::split_parse)
            .filter_map(Result::ok)
            .map(|cookie| (cookie.name().to_owned(), cookie.value().to_owned()))
            .collect();

        Arc::new(CookieJar {
            incoming,
            #[allow(
                clippy::disallowed_types,
                reason = "ver o campo `pending` do CookieJar"
            )]
            pending: std::sync::Mutex::new(Vec::new()),
        })
    }

    /// Roda `future` com este jar instalado na tarefa.
    pub(super) async fn scope<F: Future>(jar: Arc<CookieJar>, future: F) -> F::Output {
        CURRENT.scope(jar, future).await
    }

    /// O que o handler escreveu, para o layer carimbar na resposta.
    ///
    /// `unwrap_or_else(into_inner)` em vez de `unwrap`: um handler que entrou em
    /// pânico segurando o lock envenena o mutex, e perder os cookies por causa
    /// disso trocaria um `500` por um `500` mais uma sessão corrompida.
    pub(super) fn drain(jar: &CookieJar) -> Vec<Cookie<'static>> {
        std::mem::take(
            &mut *jar
                .pending
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
    }

    /// Monta o cookie com a política que vale para todos.
    ///
    /// `HttpOnly` sempre. É o que impede um XSS de ler o token por JavaScript —
    /// a diferença entre um script injetado poder incomodar o usuário e poder
    /// roubar a sessão dele inteira.
    fn build(name: CookieName, value: &str, ttl: std::time::Duration) -> Cookie<'static> {
        let seconds = i64::try_from(ttl.as_secs()).unwrap_or(i64::MAX);

        Cookie::build((name.as_str(), value.to_owned()))
            .path("/")
            .max_age(cookie::time::Duration::seconds(seconds))
            .http_only(true)
            .secure(SessionPolicy::SECURE)
            .same_site(SessionPolicy::SAME_SITE)
            .build()
    }

    /// Enfileira um cookie para sair na resposta.
    fn push(cookie: Cookie<'static>) -> Result<(), ApiError> {
        Self::jar()?
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(cookie);

        Ok(())
    }

    /// O jar desta tarefa.
    ///
    /// Falha fora do escopo em vez de descartar o cookie em silêncio. Descartar
    /// seria um login que responde `200` sem logar ninguém — o modo de falha
    /// mais caro que este desenho pode ter.
    fn jar() -> Result<Arc<CookieJar>, ApiError> {
        CURRENT.try_with(Arc::clone).map_err(|_| {
            SystemLogger::get().error(
                "o middleware de cookie não executou: a ordem dos layers do router está errada",
                [],
            );

            ApiError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "cookies indisponíveis",
            )
        })
    }

    /// Quanto tempo o cookie deste nome vale.
    const fn ttl_of(name: CookieName) -> std::time::Duration {
        match name {
            CookieName::Access => SessionPolicy::ACCESS_TTL,
            CookieName::Refresh => SessionPolicy::REFRESH_TTL,
        }
    }
}

impl CookiePort for CookieContext {
    fn read(&self, name: CookieName) -> Result<Option<String>, ApiError> {
        Ok(Self::jar()?
            .incoming
            .iter()
            .find(|(key, _)| key == name.as_str())
            .map(|(_, value)| value.clone())
            .filter(|value| !value.is_empty()))
    }

    fn set(&self, name: CookieName, value: &str) -> Result<(), ApiError> {
        Self::push(Self::build(name, value, Self::ttl_of(name)))
    }

    /// Apagar é emitir o mesmo cookie já vencido.
    ///
    /// Reaproveitar o [`Self::build`] fecha a chance de o cookie de logout sair
    /// com `Path` ou `SameSite` diferente do de login — caso em que o navegador
    /// guarda os dois e a sessão não morre.
    fn clear(&self, name: CookieName) -> Result<(), ApiError> {
        Self::push(Self::build(name, "", std::time::Duration::ZERO))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use pretty_assertions::assert_eq;

    fn headers(cookie: &'static str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::COOKIE, HeaderValue::from_static(cookie));

        headers
    }

    #[tokio::test]
    async fn o_cookie_apresentado_e_encontrado_entre_outros() {
        let jar = CookieContext::open(&headers("theme=dark; auth_token=abc123; other=1"));

        CookieContext::scope(jar, async {
            assert_eq!(
                CookieContext.read(CookieName::Access).expect("há escopo"),
                Some("abc123".to_owned())
            );
        })
        .await;
    }

    /// É o que um `Max-Age=0` deixa para trás em alguns clientes; tratá-lo como
    /// valor faria o logout parecer não ter funcionado.
    #[tokio::test]
    async fn cookie_vazio_conta_como_ausente() {
        let jar = CookieContext::open(&headers("auth_token="));

        CookieContext::scope(jar, async {
            assert_eq!(
                CookieContext.read(CookieName::Access).expect("há escopo"),
                None
            );
        })
        .await;
    }

    #[tokio::test]
    async fn o_que_o_handler_escreve_o_layer_recolhe() {
        let jar = CookieContext::open(&HeaderMap::new());
        let closing = Arc::clone(&jar);

        CookieContext::scope(jar, async {
            CookieContext
                .set(CookieName::Access, "t")
                .expect("há escopo");
            CookieContext.clear(CookieName::Refresh).expect("há escopo");
        })
        .await;

        let drained = CookieContext::drain(&closing);

        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].value(), "t");
        assert_eq!(drained[1].value(), "");
        assert_eq!(drained[1].max_age(), Some(cookie::time::Duration::ZERO));
    }

    /// Apagar tem que produzir o mesmo cookie que emitir, menos o valor: se o
    /// `Path` ou o `SameSite` divergirem, o navegador guarda os dois.
    #[tokio::test]
    async fn o_cookie_que_apaga_casa_com_o_que_emite() {
        let jar = CookieContext::open(&HeaderMap::new());
        let closing = Arc::clone(&jar);

        CookieContext::scope(jar, async {
            CookieContext
                .set(CookieName::Access, "t")
                .expect("há escopo");
            CookieContext.clear(CookieName::Access).expect("há escopo");
        })
        .await;

        let drained = CookieContext::drain(&closing);

        assert_eq!(drained[0].name(), drained[1].name());
        assert_eq!(drained[0].path(), drained[1].path());
        assert_eq!(drained[0].same_site(), drained[1].same_site());
        assert_eq!(drained[0].secure(), drained[1].secure());
    }

    /// `HttpOnly` é o que separa um XSS que incomoda de um que rouba a sessão.
    #[tokio::test]
    async fn o_cookie_de_sessao_e_http_only() {
        let jar = CookieContext::open(&HeaderMap::new());
        let closing = Arc::clone(&jar);

        CookieContext::scope(jar, async {
            CookieContext
                .set(CookieName::Access, "t")
                .expect("há escopo");
        })
        .await;

        let drained = CookieContext::drain(&closing);

        assert_eq!(drained[0].http_only(), Some(true));
        assert_eq!(drained[0].secure(), Some(SessionPolicy::SECURE));
        assert_eq!(drained[0].same_site(), Some(SessionPolicy::SAME_SITE));
    }

    /// Descartar o cookie em silêncio seria um login que responde 200 sem
    /// logar ninguém.
    #[tokio::test]
    async fn fora_do_escopo_escrever_um_cookie_e_erro() {
        assert_eq!(
            CookieContext
                .set(CookieName::Access, "t")
                .err()
                .map(|e| e.status()),
            Some(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        );
    }
}
