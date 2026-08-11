//! O acumulador que a chain preenche, um grupo por elo.

use portmaster_app::{AppSecrets, DomainSecrets, InfraSecrets};

use crate::config::api_config::ApiConfig;
use crate::config::jwt_config::JwtConfig;
use crate::config::secrets::Secrets;

/// O estado meio construído da configuração, enquanto a chain roda.
///
/// Deliberadamente **não** é a [`Secrets`]: aquela é imutável e tem que estar
/// completa. Esta é a metade do caminho, e existe só entre o primeiro elo e o
/// [`Self::into_secrets`] — depois disso nem o draft nem a chain existem em
/// memória.
///
/// Cada slot pertence a exatamente um elo. É o que dispensa ordem: os elos são
/// independentes, ninguém lê o slot de ninguém, e acrescentar um grupo é
/// acrescentar um campo aqui e um arquivo em
/// [`chain`](crate::config::chain).
#[derive(Debug, Default)]
pub(crate) struct BootDraft {
    /// Onde o servidor escuta, preenchido pelo `ApiChain`.
    pub(crate) api: Option<ApiConfig>,
    /// Token e cookies, preenchido pelo `JwtChain`.
    pub(crate) jwt: Option<JwtConfig>,
    /// Identidade de deploy, preenchida pelo `DomainChain`.
    pub(crate) domain: Option<DomainSecrets>,
    /// Conexão com o banco, preenchida pelo `DatabaseChain`.
    pub(crate) infra: Option<InfraSecrets>,
}

impl BootDraft {
    /// Congela o rascunho na configuração definitiva.
    ///
    /// Um slot vazio aqui não é variável de ambiente faltando — quem lida com
    /// isso é o [`EnvSource`](crate::config::env_source::EnvSource), e todo elo
    /// preenche o seu slot com padrões quando não acha nada. É elo que não
    /// rodou, ou seja, um `#[distributed_slice]` que ninguém declarou.
    pub(crate) fn into_secrets(self) -> anyhow::Result<Secrets> {
        Ok(Secrets {
            api: self.api.ok_or_else(|| missing("api"))?,
            jwt: self.jwt.ok_or_else(|| missing("jwt"))?,
            app: AppSecrets {
                domain: self.domain.ok_or_else(|| missing("domain"))?,
                infra: self.infra.ok_or_else(|| missing("infra"))?,
            },
        })
    }
}

/// A recusa por um grupo que elo nenhum preencheu.
fn missing(group: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "o grupo de configuração \"{group}\" nunca foi preenchido: falta o elo dele em CONFIG_LINKS"
    )
}
