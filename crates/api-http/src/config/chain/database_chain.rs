//! O elo que lê a conexão com o banco.

use linkme::distributed_slice;
use portmaster_app::{DatabaseSslMode, InfraSecrets, SecretString};

use crate::config::boot_draft::BootDraft;
use crate::config::config_link::ConfigLink;
use crate::config::config_links::CONFIG_LINKS;
use crate::config::env::Env;
use crate::config::env_source::EnvSource;

/// Lê a conexão com o banco para um [`InfraSecrets`].
pub(crate) struct DatabaseChain;

impl DatabaseChain {
    /// Preenche o slot `infra` do rascunho.
    fn read(env: &mut EnvSource, draft: &mut BootDraft) {
        draft.infra = Some(InfraSecrets {
            database_uri: SecretString::from(Self::uri(env)),
            ssl_mode: Self::ssl_mode(env),
            ssl_ca_path: Some(env.string(Env::DB_SSL_CA, "")).filter(|path| !path.is_empty()),
            ssl_verify_hostname: env.flag(Env::DB_SSL_VERIFY_CN, true),
        });
    }

    /// Monta a URI de conexão a partir das partes.
    ///
    /// Partes soltas e não uma URI inteira porque é assim que o `docker-compose`
    /// e o PHP já as expõem — trocar por uma variável única obrigaria quem faz
    /// deploy a reescrever a configuração para ganhar nada.
    fn uri(env: &mut EnvSource) -> String {
        let user = env.required(Env::DB_USER);
        let password = env.required(Env::DB_PASSWORD);
        let name = env.required(Env::DB_NAME);
        let host = env.string(Env::DB_HOST, "127.0.0.1");
        let port: u16 = env.number(Env::DB_PORT, 3306);

        format!("mysql://{user}:{password}@{host}:{port}/{name}")
    }

    /// Traduz o modo de TLS pedido.
    ///
    /// Um modo irreconhecível é queixa e não queda no padrão: cair em `disabled`
    /// conectaria em claro um deploy que pediu TLS, e ninguém veria.
    fn ssl_mode(env: &mut EnvSource) -> DatabaseSslMode {
        let raw = env.string(Env::DB_SSL_MODE, "");

        match raw.to_ascii_lowercase().as_str() {
            "" | "disabled" | "disable" | "off" => DatabaseSslMode::Disabled,
            "required" | "require" | "on" => DatabaseSslMode::Required,
            "verify_ca" | "verify-ca" => DatabaseSslMode::VerifyCa,
            other => {
                env.refuse(format!(
                    "{} não reconhece `{other}`: use disabled, required ou verify_ca",
                    Env::DB_SSL_MODE
                ));

                DatabaseSslMode::Disabled
            }
        }
    }
}

/// A conexão com o banco é um grupo da chain deste binário.
#[allow(
    unsafe_code,
    reason = "o #[distributed_slice] expande para um static com link_section; o desvio fica no registro, e não sobe para o lib.rs"
)]
#[distributed_slice(CONFIG_LINKS)]
static DATABASE: ConfigLink = ConfigLink {
    read: DatabaseChain::read,
};

#[cfg(test)]
#[path = "tests/database_chain_test.rs"]
mod tests;
