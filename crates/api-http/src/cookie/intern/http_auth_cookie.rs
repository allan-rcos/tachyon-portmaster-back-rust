//! Os cookies de sessão sobre o crate `cookie`. Não sai do módulo.

use axum::http::{header, HeaderMap};
use cookie::{Cookie, SameSite};

use crate::config::jwt_config::JwtConfig;
use crate::cookie::auth_cookie::AuthCookie;

/// Emite e lê os cookies de sessão.
///
/// Construídos pelo crate `cookie`, e não por `format!`. O que se ganha não é
/// concisão: é `SameSite` virar um enum em vez de uma string que ninguém valida,
/// `Max-Age` virar uma duração tipada, e a leitura do cabeçalho `Cookie` deixar
/// de ser um parser nosso — que tratava `a=1; b=2` bem e o resto da gramática
/// não.
#[derive(Clone)]
pub(crate) struct HttpAuthCookie {
    /// Nome do cookie do access token.
    access_name: String,
    /// Nome do cookie do refresh token.
    refresh_name: String,
    /// Validade do cookie de access.
    access_max_age: cookie::time::Duration,
    /// Validade do cookie de refresh.
    refresh_max_age: cookie::time::Duration,
    /// Se os cookies exigem HTTPS.
    secure: bool,
    /// A política `SameSite` dos cookies.
    same_site: SameSite,
}

impl HttpAuthCookie {
    /// Monta a partir da configuração.
    ///
    /// A configuração entra pelo construtor e não fica: o que sobra são os
    /// valores já resolvidos, e o [`JwtConfig`] é descartado por quem o
    /// emprestou.
    pub(crate) fn new(config: &JwtConfig) -> Self {
        Self {
            access_name: config.cookie_name.clone(),
            refresh_name: config.refresh_cookie_name.clone(),
            access_max_age: cookie::time::Duration::seconds(as_seconds(config.ttl)),
            refresh_max_age: cookie::time::Duration::seconds(as_seconds(config.refresh_ttl)),
            secure: config.cookie_secure,
            same_site: same_site_of(&config.cookie_same_site),
        }
    }

    /// Monta o cookie com a política que vale para os dois.
    fn build(&self, name: &str, value: &str, max_age: cookie::time::Duration) -> Cookie<'static> {
        Cookie::build((name.to_owned(), value.to_owned()))
            .path("/")
            .max_age(max_age)
            .http_only(true)
            .secure(self.secure)
            .same_site(self.same_site)
            .build()
    }

    /// Procura um cookie pelo nome no cabeçalho `Cookie`.
    ///
    /// Um cookie presente e vazio conta como ausente: é o que um `Max-Age=0`
    /// deixa para trás em alguns clientes, e tratá-lo como valor faria o
    /// logout parecer não ter funcionado.
    fn read(headers: &HeaderMap, name: &str) -> Option<String> {
        headers
            .get_all(header::COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .flat_map(Cookie::split_parse)
            .filter_map(Result::ok)
            .find(|cookie| cookie.name() == name)
            .map(|cookie| cookie.value().to_owned())
            .filter(|value| !value.is_empty())
    }
}

impl AuthCookie for HttpAuthCookie {
    fn issue_access(&self, token: &str) -> Cookie<'static> {
        self.build(&self.access_name, token, self.access_max_age)
    }

    fn issue_refresh(&self, token: &str) -> Cookie<'static> {
        self.build(&self.refresh_name, token, self.refresh_max_age)
    }

    fn clear_access(&self) -> Cookie<'static> {
        self.build(&self.access_name, "", cookie::time::Duration::ZERO)
    }

    fn clear_refresh(&self) -> Cookie<'static> {
        self.build(&self.refresh_name, "", cookie::time::Duration::ZERO)
    }

    fn read_access(&self, headers: &HeaderMap) -> Option<String> {
        Self::read(headers, &self.access_name)
    }

    fn read_refresh(&self, headers: &HeaderMap) -> Option<String> {
        Self::read(headers, &self.refresh_name)
    }
}

/// A duração em segundos, saturada na faixa do `cookie`.
fn as_seconds(ttl: std::time::Duration) -> i64 {
    i64::try_from(ttl.as_secs()).unwrap_or(i64::MAX)
}

/// A política `SameSite` que o nome descreve.
///
/// Um valor irreconhecível vira `Lax` — o padrão dos navegadores, e o que menos
/// surpreende. `None` sem `Secure` é recusado pelos navegadores de qualquer
/// forma, então não vale tratá-lo como caso especial aqui.
fn same_site_of(name: &str) -> SameSite {
    match name.to_ascii_lowercase().as_str() {
        "strict" => SameSite::Strict,
        "none" => SameSite::None,
        _ => SameSite::Lax,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use pretty_assertions::assert_eq;
    use secrecy::SecretString;
    use std::time::Duration;

    fn config(secure: bool, same_site: &str) -> JwtConfig {
        JwtConfig {
            secret: SecretString::from("um-segredo-de-pelo-menos-32-bytes!!"),
            ttl: Duration::from_secs(900),
            issuer: "portmaster".to_owned(),
            cookie_name: "auth_token".to_owned(),
            cookie_secure: secure,
            cookie_same_site: same_site.to_owned(),
            refresh_cookie_name: "refresh_token".to_owned(),
            refresh_ttl: Duration::from_secs(604_800),
        }
    }

    /// `HttpOnly` é o que separa um XSS que incomoda de um que rouba a sessão.
    #[test]
    fn o_cookie_de_sessao_e_http_only() {
        let cookie = HttpAuthCookie::new(&config(true, "lax")).issue_access("jwt.de.mentira");

        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.secure(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    /// Em desenvolvimento o servidor é HTTP puro, e um cookie `Secure` nunca
    /// chegaria — a sessão simplesmente não funcionaria localmente.
    #[test]
    fn secure_desligado_sai_do_cookie() {
        let cookie = HttpAuthCookie::new(&config(false, "lax")).issue_access("t");

        assert_eq!(cookie.secure(), Some(false));
    }

    #[test]
    fn limpar_expira_o_cookie_na_hora() {
        let cookie = HttpAuthCookie::new(&config(true, "strict")).clear_refresh();

        assert_eq!(cookie.value(), "");
        assert_eq!(cookie.max_age(), Some(cookie::time::Duration::ZERO));
    }

    #[test]
    fn o_cookie_apresentado_e_encontrado_entre_outros() {
        let cookies = HttpAuthCookie::new(&config(true, "lax"));
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("theme=dark; auth_token=abc123; other=1"),
        );

        assert_eq!(cookies.read_access(&headers), Some("abc123".to_owned()));
    }

    /// É o que um `Max-Age=0` deixa para trás em alguns clientes; tratá-lo como
    /// valor faria o logout parecer não ter funcionado.
    #[test]
    fn cookie_vazio_conta_como_ausente() {
        let cookies = HttpAuthCookie::new(&config(true, "lax"));
        let mut headers = HeaderMap::new();
        headers.insert(header::COOKIE, HeaderValue::from_static("auth_token="));

        assert_eq!(cookies.read_access(&headers), None);
    }
}
