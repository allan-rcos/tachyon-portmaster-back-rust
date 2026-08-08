//! O token e os cookies que o carregam.

use std::time::Duration;

use portmaster_app::SecretString;

/// O token e os cookies que o carregam.
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
