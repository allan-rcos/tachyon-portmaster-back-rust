//! Os testes de `mariadb_pool`.

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
    let error =
        connect_options(&secrets(DatabaseSslMode::VerifyCa, None)).expect_err("deveria recusar");

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
