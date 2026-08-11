//! O elo que lê o token de sessão e os cookies que o carregam.

use linkme::distributed_slice;
use portmaster_app::SecretString;

use crate::config::boot_draft::BootDraft;
use crate::config::config_link::ConfigLink;
use crate::config::config_links::CONFIG_LINKS;
use crate::config::env::Env;
use crate::config::env_source::EnvSource;
use crate::config::jwt_config::JwtConfig;

/// O mínimo que o HS256 aceita como segredo, em bytes.
///
/// A assinatura é produzida com o segredo cru, então uma chave menor que o
/// digest de 256 bits enfraquece o que ela assina: dá para enumerar o espaço de
/// chaves mais rápido do que forjar o hash. É a falha que a `firebase/php-jwt`
/// fechou na 7.0 (CVE-2025-45769) recusando a chave de saída.
const MIN_SECRET_BYTES: usize = 32;

/// Lê as variáveis do token de sessão para um [`JwtConfig`].
///
/// O único elo que recusa um **valor**, e não só a leitura dele: um segredo
/// fraco não é um número a corrigir, é motivo para não atender requisição
/// nenhuma. A biblioteca só reclamaria no primeiro token emitido, o que apareceria
/// como um 500 no login sem dizer nada sobre o ambiente.
///
/// O PHP lançava exceção aqui, saindo da chain no meio. Este registra a queixa e
/// deixa os outros elos rodarem: o boot cai do mesmo jeito, e quem o vê cair
/// descobre de uma vez tudo o que precisa arrumar.
pub(crate) struct JwtChain;

impl JwtChain {
    /// Preenche o slot `jwt` do rascunho.
    fn read(env: &mut EnvSource, draft: &mut BootDraft) {
        let defaults = JwtConfig::default();
        let secret = env.required(Env::JWT_SECRET);

        if !secret.is_empty() && secret.len() < MIN_SECRET_BYTES {
            env.refuse(format!(
                "{} precisa de ao menos {MIN_SECRET_BYTES} bytes: um segredo menor que o digest \
                 do HS256 não acrescenta entropia à assinatura",
                Env::JWT_SECRET
            ));
        }

        draft.jwt = Some(JwtConfig {
            secret: SecretString::from(secret),
            issuer: env.string(Env::JWT_ISSUER, &defaults.issuer),
        });
    }
}

/// O token de sessão é um grupo da chain deste binário.
#[allow(
    unsafe_code,
    reason = "o #[distributed_slice] expande para um static com link_section; o desvio fica no registro, e não sobe para o lib.rs"
)]
#[distributed_slice(CONFIG_LINKS)]
static JWT: ConfigLink = ConfigLink {
    read: JwtChain::read,
};
