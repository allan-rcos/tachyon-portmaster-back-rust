//! O construtor da camada.

use crate::bootstrap::domain_provider::DomainProviderImpl;
use crate::bootstrap::provider::DomainProvider;
use crate::config::DomainSecrets;

/// Inicializa o `domain` e devolve o seu provider.
///
/// Não depende de camada nenhuma, então é o fim da cadeia de `register`: quem o
/// chama é o `app`, antes de montar os seus casos de uso.
pub fn register(secrets: DomainSecrets) -> impl DomainProvider {
    DomainProviderImpl::new(secrets)
}
