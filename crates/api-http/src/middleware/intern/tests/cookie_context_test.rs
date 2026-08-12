//! Os testes de `cookie_context`.

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
