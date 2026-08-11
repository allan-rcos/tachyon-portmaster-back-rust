//! O token e os cookies que o carregam.

use std::time::Duration;

use portmaster_app::SecretString;

/// O token e os cookies que o carregam.
///
/// Os padrões moram aqui pela mesma razão que os da
/// [`ApiConfig`](crate::config::api_config::ApiConfig): o elo que lê o ambiente
/// usa cada campo do `Self::default()` como fallback, e não reescreve nenhum.
///
/// O `secret` é a exceção — o padrão dele é vazio de propósito, porque não
/// existe padrão seguro para um segredo de assinatura. Quem o exige é o
/// [`JwtChain`](crate::config::chain::jwt_chain::JwtChain).
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Segredo de assinatura HS256.
    pub secret: SecretString,
    /// Validade do access token.
    pub ttl: Duration,
    /// Emissor, gravado e conferido na claim `iss`.
    pub issuer: String,
    /// Nome do cookie do access token.
    pub cookie_name: String,
    /// Se os cookies exigem HTTPS.
    pub cookie_secure: bool,
    /// Política `SameSite` dos cookies.
    pub cookie_same_site: String,
    /// Nome do cookie do refresh token.
    pub refresh_cookie_name: String,
    /// Validade do refresh token.
    pub refresh_ttl: Duration,
}

impl Default for JwtConfig {
    /// Os valores que o PHP usava, menos o segredo — que não tem padrão.
    fn default() -> Self {
        Self {
            secret: SecretString::from(String::new()),
            ttl: Duration::from_secs(3600),
            issuer: "tachyon/portmaster".to_owned(),
            cookie_name: "auth_token".to_owned(),
            cookie_secure: false,
            cookie_same_site: "Strict".to_owned(),
            refresh_cookie_name: "refresh_token".to_owned(),
            // Quatorze dias, como o PHP.
            refresh_ttl: Duration::from_secs(1_209_600),
        }
    }
}
