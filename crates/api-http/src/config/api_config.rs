//! Onde o servidor escuta e como se comporta.

use std::time::Duration;

use crate::config::jwt_config::JwtConfig;

/// Onde o servidor escuta e como se comporta.
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Endereço de escuta.
    pub host: String,
    /// Porta de escuta.
    pub port: u16,
    /// Nome do ambiente, para o `/info`.
    pub environment: String,
    /// Teto de tempo de uma requisição.
    pub request_timeout: Duration,
    /// Origens aceitas pelo CORS; vazio libera qualquer uma.
    pub cors_origins: Vec<String>,
    /// Tudo que diz respeito a token e cookie.
    pub jwt: JwtConfig,
}
