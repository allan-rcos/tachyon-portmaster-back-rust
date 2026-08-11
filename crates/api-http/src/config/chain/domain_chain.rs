//! O elo que lê quem é este servidor na composição do Snowflake.

use linkme::distributed_slice;
use portmaster_app::DomainSecrets;

use crate::config::boot_draft::BootDraft;
use crate::config::config_link::ConfigLink;
use crate::config::config_links::CONFIG_LINKS;
use crate::config::env::Env;
use crate::config::env_source::EnvSource;

/// Lê a identidade de deploy para um [`DomainSecrets`].
///
/// Zero e zero são os padrões porque um deploy de uma instância só é o caso
/// comum, e nele não há com quem colidir. O que quebra é subir duas instâncias
/// sem diferenciá-las — e aí os ids se sobrepõem, que é a razão de as duas
/// variáveis existirem.
pub(crate) struct DomainChain;

impl DomainChain {
    /// Preenche o slot `domain` do rascunho.
    fn read(env: &mut EnvSource, draft: &mut BootDraft) {
        draft.domain = Some(DomainSecrets {
            cluster_id: env.number(Env::CLUSTER_ID, 0),
            server_id: env.number(Env::SERVER_ID, 0),
        });
    }
}

/// A identidade de deploy é um grupo da chain deste binário.
#[allow(
    unsafe_code,
    reason = "o #[distributed_slice] expande para um static com link_section; o desvio fica no registro, e não sobe para o lib.rs"
)]
#[distributed_slice(CONFIG_LINKS)]
static DOMAIN: ConfigLink = ConfigLink {
    read: DomainChain::read,
};
