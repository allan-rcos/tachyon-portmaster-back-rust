//! Os cookies de sessão sobre o crate `cookie`. Não sai do módulo.

use axum::http::{header, HeaderMap};
use cookie::Cookie;

use crate::ports::cookie::auth_cookie::AuthCookie;
use crate::ports::cookie::cookie_name::CookieName;
use crate::ports::session_policy::SessionPolicy;

/// Emite e lê os cookies de sessão.
///
/// Construídos pelo crate `cookie`, e não por `format!`. O que se ganha não é
/// concisão: é `SameSite` virar um enum em vez de uma string que ninguém valida,
/// `Max-Age` virar uma duração tipada, e a leitura do cabeçalho `Cookie` deixar
/// de ser um parser nosso — que tratava `a=1; b=2` bem e o resto da gramática
/// não.
///
/// Não guarda nada. Nome, validade, `Secure` e `SameSite` são decididos em
/// compilação pela [`SessionPolicy`] e pelo [`CookieName`]; antes eram seis
/// campos copiados da configuração, e seis variáveis de ambiente que podiam
/// discordar entre si.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct HttpAuthCookie;

impl HttpAuthCookie {
    /// Monta o emissor de cookies.
    pub(crate) const fn new() -> Self {
        Self
    }

    /// Monta o cookie com a política que vale para os dois.
    fn build(name: CookieName, value: &str, max_age: std::time::Duration) -> Cookie<'static> {
        Cookie::build((name.as_str(), value.to_owned()))
            .path("/")
            .max_age(as_cookie_duration(max_age))
            .http_only(true)
            .secure(SessionPolicy::SECURE)
            .same_site(SessionPolicy::SAME_SITE)
            .build()
    }

    /// O cookie que apaga um nome.
    ///
    /// Reaproveita o [`Self::build`] com valor vazio e validade zero: apagar é
    /// emitir o mesmo cookie já vencido, e escrever isso à parte abriria a chance
    /// de o cookie de logout sair com `Path` ou `SameSite` diferente do de login
    /// — caso em que o navegador guarda os dois e a sessão não morre.
    fn clear(name: CookieName) -> Cookie<'static> {
        Self::build(name, "", std::time::Duration::ZERO)
    }

    /// Procura um cookie pelo nome no cabeçalho `Cookie`.
    ///
    /// Um cookie presente e vazio conta como ausente: é o que um `Max-Age=0`
    /// deixa para trás em alguns clientes, e tratá-lo como valor faria o
    /// logout parecer não ter funcionado.
    fn read(headers: &HeaderMap, name: CookieName) -> Option<String> {
        headers
            .get_all(header::COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .flat_map(Cookie::split_parse)
            .filter_map(Result::ok)
            .find(|cookie| cookie.name() == name.as_str())
            .map(|cookie| cookie.value().to_owned())
            .filter(|value| !value.is_empty())
    }
}

impl AuthCookie for HttpAuthCookie {
    fn issue_access(&self, token: &str) -> Cookie<'static> {
        Self::build(CookieName::Access, token, SessionPolicy::ACCESS_TTL)
    }

    fn issue_refresh(&self, token: &str) -> Cookie<'static> {
        Self::build(CookieName::Refresh, token, SessionPolicy::REFRESH_TTL)
    }

    fn clear_access(&self) -> Cookie<'static> {
        Self::clear(CookieName::Access)
    }

    fn clear_refresh(&self) -> Cookie<'static> {
        Self::clear(CookieName::Refresh)
    }

    fn read_access(&self, headers: &HeaderMap) -> Option<String> {
        Self::read(headers, CookieName::Access)
    }

    fn read_refresh(&self, headers: &HeaderMap) -> Option<String> {
        Self::read(headers, CookieName::Refresh)
    }
}

/// A duração em segundos, saturada na faixa do crate `cookie`.
fn as_cookie_duration(ttl: std::time::Duration) -> cookie::time::Duration {
    cookie::time::Duration::seconds(i64::try_from(ttl.as_secs()).unwrap_or(i64::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use pretty_assertions::assert_eq;

    /// `HttpOnly` é o que separa um XSS que incomoda de um que rouba a sessão.
    #[test]
    fn o_cookie_de_sessao_e_http_only() {
        let cookie = HttpAuthCookie::new().issue_access("jwt.de.mentira");

        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.secure(), Some(SessionPolicy::SECURE));
        assert_eq!(cookie.same_site(), Some(SessionPolicy::SAME_SITE));
    }

    /// Em desenvolvimento o servidor é HTTP puro, e um cookie `Secure` nunca
    /// chegaria — a sessão simplesmente não funcionaria localmente.
    #[test]
    fn secure_segue_o_perfil_de_compilacao() {
        assert_eq!(
            HttpAuthCookie::new().issue_access("t").secure(),
            Some(!cfg!(debug_assertions))
        );
    }

    #[test]
    fn limpar_expira_o_cookie_na_hora() {
        let cookie = HttpAuthCookie::new().clear_refresh();

        assert_eq!(cookie.value(), "");
        assert_eq!(cookie.max_age(), Some(cookie::time::Duration::ZERO));
    }

    /// Apagar tem que produzir o mesmo cookie que emitir, menos o valor: se o
    /// `Path` ou o `SameSite` divergirem, o navegador guarda os dois.
    #[test]
    fn o_cookie_que_apaga_casa_com_o_que_emite() {
        let issued = HttpAuthCookie::new().issue_access("t");
        let cleared = HttpAuthCookie::new().clear_access();

        assert_eq!(cleared.name(), issued.name());
        assert_eq!(cleared.path(), issued.path());
        assert_eq!(cleared.same_site(), issued.same_site());
        assert_eq!(cleared.secure(), issued.secure());
    }

    #[test]
    fn o_cookie_apresentado_e_encontrado_entre_outros() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("theme=dark; auth_token=abc123; other=1"),
        );

        assert_eq!(
            HttpAuthCookie::new().read_access(&headers),
            Some("abc123".to_owned())
        );
    }

    /// É o que um `Max-Age=0` deixa para trás em alguns clientes; tratá-lo como
    /// valor faria o logout parecer não ter funcionado.
    #[test]
    fn cookie_vazio_conta_como_ausente() {
        let mut headers = HeaderMap::new();
        headers.insert(header::COOKIE, HeaderValue::from_static("auth_token="));

        assert_eq!(HttpAuthCookie::new().read_access(&headers), None);
    }
}
