//! Tudo que o processo precisa para subir.

use std::time::Duration;

use anyhow::Context;
use portmaster_app::{AppSecrets, DatabaseSslMode, DomainSecrets, InfraSecrets, SecretString};

use crate::config::api_config::ApiConfig;
use crate::config::env::Env;
use crate::config::jwt_config::JwtConfig;

/// O mínimo que o HS256 aceita como segredo.
///
/// Um segredo menor que o digest não acrescenta entropia à assinatura: dá para
/// enumerar o espaço de chaves mais rápido do que forjar o hash.
const MIN_JWT_SECRET_BYTES: usize = 32;

/// Tudo que o processo precisa para subir.
#[derive(Debug, Clone)]
pub struct Secrets {
    /// O que é desta camada.
    pub api: ApiConfig,
    /// O que é das camadas de baixo.
    pub app: AppSecrets,
}

impl Secrets {
    /// Lê o ambiente e monta a configuração de todas as camadas.
    ///
    /// Falta de segredo obrigatório derruba o boot. É deliberado: um sistema que
    /// sobe com o JWT num valor padrão aceita tokens que qualquer um pode
    /// forjar, e descobrir isso em produção é caro demais comparado a não subir.
    pub fn load() -> anyhow::Result<Self> {
        let jwt_secret = required(Env::JWT_SECRET)?;

        anyhow::ensure!(
            jwt_secret.len() >= MIN_JWT_SECRET_BYTES,
            "{} precisa de ao menos {MIN_JWT_SECRET_BYTES} bytes: um segredo menor que o \
             digest do HS256 não acrescenta entropia à assinatura",
            Env::JWT_SECRET
        );

        Ok(Self {
            api: ApiConfig {
                host: optional(Env::HOST).unwrap_or_else(|| "127.0.0.1".to_owned()),
                port: parsed(Env::PORT, 8000)?,
                environment: optional(Env::ENVIRONMENT).unwrap_or_else(|| "development".to_owned()),
                request_timeout: Duration::from_secs(parsed(Env::REQUEST_TIMEOUT, 30)?),
                cors_origins: optional(Env::CORS_ORIGINS)
                    .map(|list| {
                        list.split(',')
                            .map(str::trim)
                            .filter(|origin| !origin.is_empty())
                            .map(str::to_owned)
                            .collect()
                    })
                    .unwrap_or_default(),
                jwt: JwtConfig {
                    secret: SecretString::from(jwt_secret),
                    ttl: Duration::from_secs(parsed(Env::JWT_TTL, 3600)?),
                    issuer: optional(Env::JWT_ISSUER)
                        .unwrap_or_else(|| "tachyon/portmaster".to_owned()),
                    cookie_name: optional(Env::JWT_COOKIE_NAME)
                        .unwrap_or_else(|| "auth_token".to_owned()),
                    cookie_secure: flag(Env::JWT_COOKIE_SECURE, false),
                    cookie_same_site: optional(Env::JWT_COOKIE_SAME_SITE)
                        .unwrap_or_else(|| "Strict".to_owned()),
                    refresh_cookie_name: optional(Env::REFRESH_COOKIE_NAME)
                        .unwrap_or_else(|| "refresh_token".to_owned()),
                    // Quatorze dias, como o PHP.
                    refresh_ttl: Duration::from_secs(parsed(Env::REFRESH_TTL, 1_209_600)?),
                },
            },
            app: AppSecrets {
                domain: DomainSecrets {
                    cluster_id: parsed(Env::CLUSTER_ID, 0)?,
                    server_id: parsed(Env::SERVER_ID, 0)?,
                },
                infra: InfraSecrets {
                    database_uri: SecretString::from(database_uri()?),
                    ssl_mode: ssl_mode()?,
                    ssl_ca_path: optional(Env::DB_SSL_CA),
                    ssl_verify_hostname: flag(Env::DB_SSL_VERIFY_CN, true),
                },
            },
        })
    }
}

/// Monta a URI de conexão a partir das partes.
///
/// Partes soltas e não uma URI inteira porque é assim que o `docker-compose` e o
/// PHP já as expõem — trocar por uma variável única obrigaria quem faz deploy a
/// reescrever a configuração para ganhar nada.
fn database_uri() -> anyhow::Result<String> {
    let user = required(Env::DB_USER)?;
    let password = required(Env::DB_PASSWORD)?;
    let host = optional(Env::DB_HOST).unwrap_or_else(|| "127.0.0.1".to_owned());
    let port: u16 = parsed(Env::DB_PORT, 3306)?;
    let name = required(Env::DB_NAME)?;

    Ok(format!("mysql://{user}:{password}@{host}:{port}/{name}"))
}

/// Traduz o modo de TLS pedido.
fn ssl_mode() -> anyhow::Result<DatabaseSslMode> {
    let Some(raw) = optional(Env::DB_SSL_MODE) else {
        return Ok(DatabaseSslMode::Disabled);
    };

    match raw.to_ascii_lowercase().as_str() {
        "disabled" | "disable" | "off" | "" => Ok(DatabaseSslMode::Disabled),
        "required" | "require" | "on" => Ok(DatabaseSslMode::Required),
        "verify_ca" | "verify-ca" => Ok(DatabaseSslMode::VerifyCa),
        other => anyhow::bail!(
            "{} não reconhece `{other}`: use disabled, required ou verify_ca",
            Env::DB_SSL_MODE
        ),
    }
}

/// Uma variável obrigatória.
fn required(name: &str) -> anyhow::Result<String> {
    optional(name).with_context(|| format!("a variável {name} é obrigatória e não está definida"))
}

/// Uma variável opcional; vazia conta como ausente.
fn optional(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

/// Uma variável numérica, com padrão.
///
/// Valor presente mas ilegível é **erro**, não queda no padrão: quem escreveu
/// `APP_PORT=oito mil` quis dizer alguma coisa, e subir na 8000 esconderia o
/// engano até alguém notar que o serviço está na porta errada.
fn parsed<T: std::str::FromStr>(name: &str, default: T) -> anyhow::Result<T>
where
    T::Err: std::fmt::Display,
{
    match optional(name) {
        None => Ok(default),
        Some(raw) => raw
            .parse()
            .map_err(|e| anyhow::anyhow!("{name} não é um número válido: {raw} ({e})")),
    }
}

/// Uma variável booleana, com padrão.
fn flag(name: &str, default: bool) -> bool {
    match optional(name) {
        None => default,
        Some(raw) => matches!(
            raw.to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn o_modo_de_tls_aceita_as_grafias_usuais() {
        // O mesmo modo aparece escrito de formas diferentes em compose, shell e
        // documentação; recusar por causa de um hífen seria hostilidade.
        for grafia in ["verify_ca", "verify-ca", "VERIFY_CA"] {
            temporarily(Env::DB_SSL_MODE, Some(grafia), || {
                assert_eq!(ssl_mode().unwrap(), DatabaseSslMode::VerifyCa);
            });
        }
    }

    #[test]
    fn um_modo_de_tls_desconhecido_derruba_o_boot() {
        // Cair no padrão aqui conectaria em claro um deploy que pediu TLS.
        temporarily(Env::DB_SSL_MODE, Some("mais_ou_menos"), || {
            assert!(ssl_mode().is_err());
        });
    }

    #[test]
    fn variavel_vazia_conta_como_ausente() {
        // Um compose que declara a chave sem valor não deve derrotar o padrão.
        temporarily(Env::HOST, Some("   "), || {
            assert_eq!(optional(Env::HOST), None);
        });
    }

    #[test]
    fn numero_ilegivel_e_erro_e_nao_padrao() {
        temporarily(Env::PORT, Some("oito mil"), || {
            assert!(parsed::<u16>(Env::PORT, 8000).is_err());
        });
    }

    #[test]
    fn o_booleano_so_e_verdadeiro_nas_grafias_afirmativas() {
        for afirmativo in ["1", "true", "TRUE", "yes", "on"] {
            temporarily(Env::JWT_COOKIE_SECURE, Some(afirmativo), || {
                assert!(flag(Env::JWT_COOKIE_SECURE, false));
            });
        }

        for negativo in ["0", "false", "no", "qualquer coisa"] {
            temporarily(Env::JWT_COOKIE_SECURE, Some(negativo), || {
                assert!(!flag(Env::JWT_COOKIE_SECURE, true));
            });
        }
    }

    /// Serializa os testes que mexem no ambiente.
    ///
    /// O ambiente é global ao **processo**, e o cargo roda os testes em threads
    /// paralelas: sem este lock, dois testes que escrevem a mesma variável se
    /// atropelam e o que falha é sorteado a cada execução.
    static ENV: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Roda `body` com a variável definida, e a restaura depois.
    fn temporarily(name: &str, value: Option<&str>, body: impl FnOnce()) {
        // `unwrap_or_else` em vez de `unwrap`: se um teste anterior entrou em
        // pânico segurando o lock, envenená-lo faria os outros falharem por
        // arrasto, escondendo qual foi o problema original.
        let _guard = ENV
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let previous = std::env::var(name).ok();

        match value {
            Some(value) => std::env::set_var(name, value),
            None => std::env::remove_var(name),
        }

        body();

        match previous {
            Some(value) => std::env::set_var(name, value),
            None => std::env::remove_var(name),
        }
    }
}
