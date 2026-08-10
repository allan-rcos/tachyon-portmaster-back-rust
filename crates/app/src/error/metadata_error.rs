//! O que pode dar errado num caso de uso de metadado de sistema.

use portmaster_domain::error::MetadataError as DomainMetadataError;

use crate::error::AppError;

/// As falhas do serviço de metadados.
#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    /// O que é comum a toda a camada — validação, permissão, infraestrutura.
    #[error(transparent)]
    App(#[from] AppError),
}

impl From<DomainMetadataError> for MetadataError {
    fn from(error: DomainMetadataError) -> Self {
        match error {
            DomainMetadataError::Validation(fields) => Self::App(AppError::Validation(fields)),
        }
    }
}

impl From<anyhow::Error> for MetadataError {
    fn from(error: anyhow::Error) -> Self {
        Self::App(AppError::Infra(error))
    }
}
