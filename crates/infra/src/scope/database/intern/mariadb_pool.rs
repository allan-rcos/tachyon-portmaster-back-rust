//! A abertura do pool de conexões.
//!
//! Criado uma vez no `register` e compartilhado por clone — o `MySqlPool` é um
//! `Arc` por dentro, então cada repositório carrega um ponteiro, não um pool
//! próprio. É o exemplo canônico de recurso de borda: único, externalizado, e
//! impossível de monomorfizar por consumidor.
//!
//! Quem chama é o construtor da
//! [`MariaDbUnitOfWork`](super::mariadb_unit_of_work::MariaDbUnitOfWork), e não
//! o `register` direto: o pool não tem uso fora do handle que o carrega, e um
//! `connect()` solto convidaria a que tivesse.

use crate::config::pool::POOL_MAX_CONNECTIONS;
use crate::config::{DatabaseSslMode, InfraSecrets};
use anyhow::{bail, Context};
use secrecy::ExposeSecret;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlSslMode};
use sqlx::MySqlPool;

/// Abre o pool e confirma que o banco responde.
///
/// A verificação de sanidade é deliberada: sem ela, um erro de credencial só
/// apareceria na primeira requisição, com o processo já reportado como saudável.
/// Melhor não subir do que subir quebrado.
pub(super) async fn connect(secrets: &InfraSecrets) -> anyhow::Result<MySqlPool> {
    let options = connect_options(secrets)?;

    let pool = MySqlPoolOptions::new()
        .max_connections(POOL_MAX_CONNECTIONS)
        .connect_with(options)
        .await
        .context("falha ao conectar no MariaDB")?;

    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .context("o MariaDB conectou mas não respondeu ao SELECT de sanidade")?;

    Ok(pool)
}

/// Fixa o fuso da sessão em UTC, **depois** do parse da URI.
///
/// O `sqlx` já traz `+00:00` por padrão, mas duas coisas tornam o padrão
/// insuficiente para uma regra do sistema: ele é decisão do driver e pode mudar
/// de versão, e uma `?timezone=` na URI de conexão o sobrescreveria sem que
/// nada avisasse. Fixar depois do parse fecha as duas portas.
///
/// ## O que está em jogo
///
/// As colunas são `DATETIME`, que o `MariaDB` guarda sem converter — o que entra
/// é o que sai. O fuso da sessão é quem decide o que `CURRENT_TIMESTAMP` e
/// `NOW()` valem na hora do INSERT. Com a sessão em UTC, o que o servidor grava
/// por default é o mesmo instante que o Rust grava como `DateTime<Utc>`, e a
/// leitura de volta fecha.
///
/// Sem isso, um servidor em fuso local produziria linhas com `created_at`
/// deslocado das demais colunas de tempo — e o erro só apareceria como um
/// relatório com horas erradas, meses depois.
fn pinned_to_utc(options: MySqlConnectOptions) -> MySqlConnectOptions {
    options.timezone(Some("+00:00".to_owned()))
}

/// Traduz os segredos nas opções de conexão do driver.
///
/// `verify_ca` sem o bundle da CA é recusado aqui: sem a cadeia não há o que
/// validar, e cair em silêncio para um modo mais fraco entregaria exatamente a
/// garantia que se pediu.
fn connect_options(secrets: &InfraSecrets) -> anyhow::Result<MySqlConnectOptions> {
    let uri = secrets.database_uri.expose_secret();
    let options: MySqlConnectOptions = uri
        .parse()
        .context("a URI do banco não está num formato que o driver entenda")?;

    let options = pinned_to_utc(options);

    let options = match secrets.ssl_mode {
        DatabaseSslMode::Disabled => options.ssl_mode(MySqlSslMode::Disabled),
        DatabaseSslMode::Required => options.ssl_mode(MySqlSslMode::Required),
        DatabaseSslMode::VerifyCa => {
            let Some(ca_path) = secrets.ssl_ca_path.as_deref() else {
                bail!("ssl_mode verify_ca exige o caminho do bundle da CA");
            };

            let mode = if secrets.ssl_verify_hostname {
                MySqlSslMode::VerifyIdentity
            } else {
                MySqlSslMode::VerifyCa
            };

            options.ssl_mode(mode).ssl_ca(ca_path)
        }
    };

    Ok(options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::SecretString;

    fn secrets(mode: DatabaseSslMode, ca: Option<&str>) -> InfraSecrets {
        InfraSecrets {
            database_uri: SecretString::from("mysql://root:root@localhost:3306/portmaster"),
            ssl_mode: mode,
            ssl_ca_path: ca.map(str::to_owned),
            ssl_verify_hostname: true,
        }
    }

    /// O modo pede validação de cadeia; sem cadeia configurada, a única saída
    /// honesta é falhar em vez de conectar mais fraco do que se pediu.
    #[test]
    fn verify_ca_sem_bundle_e_recusado() {
        let error = connect_options(&secrets(DatabaseSslMode::VerifyCa, None))
            .expect_err("deveria recusar");

        assert!(error.to_string().contains("bundle da CA"));
    }

    #[test]
    fn os_demais_modos_dispensam_bundle() {
        assert!(connect_options(&secrets(DatabaseSslMode::Disabled, None)).is_ok());
        assert!(connect_options(&secrets(DatabaseSslMode::Required, None)).is_ok());
    }

    #[test]
    fn uri_malformada_falha_no_boot() {
        let mut broken = secrets(DatabaseSslMode::Disabled, None);
        broken.database_uri = SecretString::from("isto não é uma uri");

        assert!(connect_options(&broken).is_err());
    }
}
