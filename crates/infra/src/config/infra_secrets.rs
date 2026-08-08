//! Os segredos que a `infra` precisa para subir.

use secrecy::SecretString;

use crate::config::DatabaseSslMode;

/// Segredos e endpoints de runtime da `infra`.
#[derive(Debug, Clone)]
pub struct InfraSecrets {
    /// URI de conexão, com a senha.
    pub database_uri: SecretString,

    /// Como tratar TLS na conexão.
    pub ssl_mode: DatabaseSslMode,

    /// Bundle da CA, lido apenas em [`DatabaseSslMode::VerifyCa`].
    pub ssl_ca_path: Option<String>,

    /// Se o nome no certificado deve casar com o host.
    pub ssl_verify_hostname: bool,
}
