//! Onde o servidor escuta e como se comporta.

use std::time::Duration;

/// Onde o servidor escuta e como se comporta.
///
/// **Os padrões moram aqui, e só aqui.** O elo que lê o ambiente
/// ([`ApiChain`](crate::config::chain::api_chain::ApiChain)) constrói um
/// `Self::default()` e usa cada campo dele como fallback da variável
/// correspondente — então um campo novo com padrão não obriga a tocar no elo, e
/// não existe um segundo lugar onde o mesmo padrão possa divergir.
///
/// O que é do token e dos cookies não está aqui: mora em
/// [`JwtConfig`](crate::config::jwt_config::JwtConfig), que é o slot de outro
/// elo. Juntá-los faria dois elos escreverem na mesma struct.
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
    /// Origens aceitas pelo CORS; vazio não libera nenhuma.
    pub cors_origins: Vec<String>,
}

impl Default for ApiConfig {
    /// Um servidor local que serve, sem precisar de variável nenhuma.
    ///
    /// O host padrão aceita só conexões locais; um contêiner que publica a porta
    /// precisa de `0.0.0.0`. E `cors_origins` vazio é o padrão seguro — é o que
    /// vale quando a API e o front saem do mesmo host.
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_owned(),
            port: 8000,
            environment: "development".to_owned(),
            request_timeout: Duration::from_secs(30),
            cors_origins: Vec::new(),
        }
    }
}
