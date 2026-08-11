//! O elo que lê onde o servidor escuta.

use linkme::distributed_slice;

use crate::config::api_config::ApiConfig;
use crate::config::boot_draft::BootDraft;
use crate::config::config_link::ConfigLink;
use crate::config::config_links::CONFIG_LINKS;
use crate::config::env::Env;
use crate::config::env_source::EnvSource;

/// Lê as variáveis do servidor HTTP para uma [`ApiConfig`].
///
/// A forma de um elo, escrita por extenso aqui e seguida por todos os outros:
/// pegue o `default()` do VO e leia cada variável usando o campo dele como
/// fallback. Os padrões ficam no VO e em nenhum outro lugar — este arquivo não
/// repete um sequer, e acrescentar um campo com padrão custa a linha que o lê.
pub(crate) struct ApiChain;

impl ApiChain {
    /// Preenche o slot `api` do rascunho.
    fn read(env: &mut EnvSource, draft: &mut BootDraft) {
        let defaults = ApiConfig::default();

        draft.api = Some(ApiConfig {
            host: env.string(Env::HOST, &defaults.host),
            port: env.number(Env::PORT, defaults.port),
            environment: env.string(Env::ENVIRONMENT, &defaults.environment),
            request_timeout: env.duration(Env::REQUEST_TIMEOUT, defaults.request_timeout),
            cors_origins: env.list(Env::CORS_ORIGINS, &defaults.cors_origins),
        });
    }
}

/// O servidor HTTP é um grupo da chain deste binário.
#[allow(
    unsafe_code,
    reason = "o #[distributed_slice] expande para um static com link_section; o desvio fica no registro, e não sobe para o lib.rs"
)]
#[distributed_slice(CONFIG_LINKS)]
static API: ConfigLink = ConfigLink {
    read: ApiChain::read,
};
