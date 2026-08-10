//! O que pode dar errado num caso de uso de produto.

use portmaster_domain::error::ProductError as DomainProductError;

use crate::error::AppError;

/// As falhas do serviço de produtos.
#[derive(Debug, thiserror::Error)]
pub enum ProductError {
    /// O produto pedido não existe.
    #[error("produto não encontrado: {0}")]
    Missing(String),

    /// O que é comum a toda a camada — validação, permissão, infraestrutura.
    #[error(transparent)]
    App(#[from] AppError),
}

impl From<DomainProductError> for ProductError {
    /// O `TableModule` de produto só recusa campo, e campo recusado é validação
    /// da camada inteira — não uma falha própria deste serviço.
    fn from(error: DomainProductError) -> Self {
        match error {
            DomainProductError::Validation(fields) => Self::App(AppError::Validation(fields)),
        }
    }
}

impl From<anyhow::Error> for ProductError {
    fn from(error: anyhow::Error) -> Self {
        Self::App(AppError::Infra(error))
    }
}

impl From<crate::error::MetadataError> for ProductError {
    /// Declarar as permissões deste serviço passa pelo serviço de metadados.
    ///
    /// O que ele recusa é slug fora de forma, e isso é validação da camada — não
    /// uma falha própria daqui. A conversão existe para que
    /// `declare_permissions` possa usar `?` sem que este erro precise conhecer o
    /// vocabulário do outro serviço.
    fn from(error: crate::error::MetadataError) -> Self {
        match error {
            crate::error::MetadataError::App(shared) => Self::App(shared),
        }
    }
}
