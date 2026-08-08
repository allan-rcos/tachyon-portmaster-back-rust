//! O que a `infra` precisa saber para subir.
//!
//! Segredo de deploy fica em [`InfraSecrets`]; decisão de arquitetura — tamanho
//! de pool, teto de cache — é constante escolhida por feature, e não passa por
//! variável de ambiente.

pub mod database_ssl_mode;
pub mod infra_secrets;

pub(crate) mod cache_limits;
pub(crate) mod pool;

pub use database_ssl_mode::DatabaseSslMode;
pub use infra_secrets::InfraSecrets;
pub use secrecy::SecretString;
