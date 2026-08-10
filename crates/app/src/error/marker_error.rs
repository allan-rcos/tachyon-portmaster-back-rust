//! O que pode dar errado ao marcar ou consultar uma marca.

use portmaster_domain::error::{MarkerError as DomainMarkerError, MetadataError};

use crate::error::AppError;

/// As falhas do serviço de marcação.
///
/// Marca não é recurso endereçável: quem pergunta por uma que não existe recebe
/// `false`, não um erro. O que sobra é campo recusado — o grupo fora de forma, o
/// valor vazio — e isso é validação.
#[derive(Debug, thiserror::Error)]
pub enum MarkerError {
    /// O que é comum a toda a camada — validação e infraestrutura.
    #[error(transparent)]
    App(#[from] AppError),
}

impl From<DomainMarkerError> for MarkerError {
    fn from(error: DomainMarkerError) -> Self {
        match error {
            DomainMarkerError::Validation(fields) => Self::App(AppError::Validation(fields)),
        }
    }
}

impl From<MetadataError> for MarkerError {
    /// O grupo de marcador é metadado de sistema, e o `TableModule` dele recusa
    /// pelo formato do slug — validação, como a do próprio marcador.
    fn from(error: MetadataError) -> Self {
        match error {
            MetadataError::Validation(fields) => Self::App(AppError::Validation(fields)),
        }
    }
}

impl From<anyhow::Error> for MarkerError {
    fn from(error: anyhow::Error) -> Self {
        Self::App(AppError::Infra(error))
    }
}
