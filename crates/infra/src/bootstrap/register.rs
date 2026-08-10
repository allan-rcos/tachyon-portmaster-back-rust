//! O construtor da camada.

use crate::bootstrap::infra_provider::InfraProviderImpl;
use crate::bootstrap::provider::InfraProvider;
use crate::config::InfraSecrets;
use crate::scope::database::intern::mariadb_unit_of_work::MariaDbUnitOfWork;

/// Inicializa a `infra` e devolve o seu provider.
///
/// Cria os recursos caros uma vez: o acesso ao banco e os quatro caches. Falhar
/// aqui derruba o boot de propósito — melhor não subir do que subir com um banco
/// inalcançável e descobrir isso na primeira requisição.
pub async fn register(secrets: InfraSecrets) -> anyhow::Result<impl InfraProvider> {
    let database = MariaDbUnitOfWork::connect(&secrets).await?;

    Ok(InfraProviderImpl::new(database))
}
