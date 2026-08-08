//! O construtor da camada.

use crate::config::InfraSecrets;
use crate::database::pool::connect;
use crate::interno::infra_provider::InfraProviderImpl;
use crate::provider::InfraProvider;

/// Inicializa a `infra` e devolve o seu provider.
///
/// Cria os recursos caros uma vez: o pool e os três caches. Falhar aqui derruba
/// o boot de propósito — melhor não subir do que subir com um banco inalcançável
/// e descobrir isso na primeira requisição.
pub async fn register(secrets: InfraSecrets) -> anyhow::Result<impl InfraProvider> {
    let pool = connect(&secrets).await?;

    Ok(InfraProviderImpl::new(pool))
}
