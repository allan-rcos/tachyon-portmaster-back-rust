//! Os segredos de todas as camadas, montados pela apresentação.

use portmaster_domain::DomainSecrets;
use portmaster_infra::config::InfraSecrets;

/// Os segredos de todas as camadas, montados pela apresentação.
///
/// O `app` os recebe inteiros e distribui: é ele que encadeia os `register` das
/// camadas de baixo, então é ele que precisa saber do que cada uma vive. A
/// apresentação lê variáveis de ambiente e preenche isto — sem conhecer
/// `domain` nem `infra`.
#[derive(Debug, Clone)]
pub struct AppSecrets {
    /// Identidade de deploy, para o gerador de Snowflake.
    pub domain: DomainSecrets,
    /// Conexão com o banco.
    pub infra: InfraSecrets,
}
