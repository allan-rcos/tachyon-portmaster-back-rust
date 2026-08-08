//! Os cookies que carregam a sessão.
//!
//! `HttpOnly` em ambos, sempre. É o que impede um XSS de ler o token por
//! JavaScript — a diferença entre um script injetado poder incomodar o usuário
//! e poder roubar a sessão dele inteira.
//!
//! `Secure` é configurável porque o ambiente de desenvolvimento roda em HTTP
//! puro, e um cookie `Secure` simplesmente não seria enviado — a sessão nunca
//! funcionaria localmente. Em produção, liga-se.

use axum::http::HeaderMap;

use crate::config::jwt_config::JwtConfig;

/// Emite e lê os cookies de sessão.
#[derive(Clone)]
pub struct AuthCookie {
    access_name: String,
    refresh_name: String,
    access_max_age: u64,
    refresh_max_age: u64,
    secure: bool,
    same_site: String,
}

impl AuthCookie {
    /// Monta a partir da configuração.
    pub(crate) fn new(config: &JwtConfig) -> Self {
        Self {
            access_name: config.cookie_name.clone(),
            refresh_name: config.refresh_cookie_name.clone(),
            access_max_age: config.ttl.as_secs(),
            refresh_max_age: config.refresh_ttl.as_secs(),
            secure: config.cookie_secure,
            same_site: config.cookie_same_site.clone(),
        }
    }

    /// O `Set-Cookie` do access token.
    pub(crate) fn issue_access(&self, token: &str) -> String {
        self.build(&self.access_name, token, self.access_max_age)
    }

    /// O `Set-Cookie` do refresh token.
    pub(crate) fn issue_refresh(&self, token: &str) -> String {
        self.build(&self.refresh_name, token, self.refresh_max_age)
    }

    /// O `Set-Cookie` que apaga o access token.
    pub(crate) fn clear_access(&self) -> String {
        self.build(&self.access_name, "", 0)
    }

    /// O `Set-Cookie` que apaga o refresh token.
    pub(crate) fn clear_refresh(&self) -> String {
        self.build(&self.refresh_name, "", 0)
    }

    /// O access token apresentado, se houver.
    pub(crate) fn read_access(&self, headers: &HeaderMap) -> Option<String> {
        read_cookie(headers, &self.access_name)
    }

    /// O refresh token apresentado, se houver.
    pub(crate) fn read_refresh(&self, headers: &HeaderMap) -> Option<String> {
        read_cookie(headers, &self.refresh_name)
    }

    /// Monta o cabeçalho.
    fn build(&self, name: &str, value: &str, max_age: u64) -> String {
        let mut cookie = format!(
            "{name}={value}; Path=/; Max-Age={max_age}; HttpOnly; SameSite={}",
            self.same_site
        );

        if self.secure {
            cookie.push_str("; Secure");
        }

        cookie
    }
}

/// Procura um cookie pelo nome no cabeçalho `Cookie`.
///
/// Escrito à mão em vez de puxar uma lib: o formato é `a=1; b=2`, o que se lê
/// não é atacante-controlado além do valor, e uma dependência a mais na borda
/// custa mais do que dez linhas.
fn read_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get_all(axum::http::header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|header| header.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value.to_owned())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::jwt_config::JwtConfig;
    use portmaster_app::SecretString;
    use pretty_assertions::assert_eq;
    use std::time::Duration;

    fn cookies(secure: bool) -> AuthCookie {
        AuthCookie::new(&JwtConfig {
            secret: SecretString::from("dev-only-change-me-32-bytes-minimum"),
            ttl: Duration::from_secs(3600),
            issuer: "tachyon/portmaster".to_owned(),
            cookie_name: "auth_token".to_owned(),
            cookie_secure: secure,
            cookie_same_site: "Strict".to_owned(),
            refresh_cookie_name: "refresh_token".to_owned(),
            refresh_ttl: Duration::from_secs(1_209_600),
        })
    }

    fn header_with(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(axum::http::header::COOKIE, value.parse().unwrap());
        headers
    }

    #[test]
    fn o_token_nunca_e_legivel_por_javascript() {
        // Sem HttpOnly, um XSS deixa de ser incômodo e passa a ser roubo de
        // sessão.
        assert!(cookies(false).issue_access("abc").contains("HttpOnly"));
        assert!(cookies(false).issue_refresh("abc").contains("HttpOnly"));
    }

    #[test]
    fn o_secure_acompanha_a_configuracao() {
        // Em desenvolvimento o cookie Secure não seria enviado sobre HTTP, e a
        // sessão nunca funcionaria localmente.
        assert!(!cookies(false).issue_access("abc").contains("Secure"));
        assert!(cookies(true).issue_access("abc").contains("Secure"));
    }

    #[test]
    fn limpar_e_um_cookie_vazio_que_expira_agora() {
        let cleared = cookies(false).clear_access();

        assert!(cleared.starts_with("auth_token=;"));
        assert!(cleared.contains("Max-Age=0"));
    }

    #[test]
    fn le_o_cookie_certo_entre_varios() {
        let headers = header_with("outro=1; auth_token=abc; refresh_token=def");

        assert_eq!(cookies(false).read_access(&headers), Some("abc".to_owned()));
        assert_eq!(
            cookies(false).read_refresh(&headers),
            Some("def".to_owned())
        );
    }

    #[test]
    fn le_cookies_espalhados_em_cabecalhos_separados() {
        // Alguns clientes mandam um `Cookie:` por par.
        let mut headers = HeaderMap::new();
        headers.append(
            axum::http::header::COOKIE,
            "auth_token=abc".parse().unwrap(),
        );
        headers.append(
            axum::http::header::COOKIE,
            "refresh_token=def".parse().unwrap(),
        );

        assert_eq!(
            cookies(false).read_refresh(&headers),
            Some("def".to_owned())
        );
    }

    #[test]
    fn cookie_vazio_conta_como_ausente() {
        // É o que o próprio `clear` deixa para trás; lê-lo como presente faria
        // o refresh tentar rodar com uma string vazia.
        let headers = header_with("auth_token=; refresh_token=");

        assert_eq!(cookies(false).read_access(&headers), None);
        assert_eq!(cookies(false).read_refresh(&headers), None);
    }

    #[test]
    fn a_ausencia_de_cabecalho_nao_e_erro() {
        assert_eq!(cookies(false).read_access(&HeaderMap::new()), None);
    }
}
