//! O que pode dar errado ao embarcar ou desembarcar carga.

use portmaster_domain::error::{FieldError, ManifestError as DomainManifestError};

use crate::error::AppError;

/// As falhas do serviço de manifesto.
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    /// O contêiner pedido não existe.
    #[error("contêiner não encontrado: {0}")]
    MissingContainer(String),

    /// O produto pedido não existe.
    #[error("produto não encontrado: {0}")]
    MissingProduct(String),

    /// A operação contradiz o estado do pátio — contêiner fechado, carga além
    /// da capacidade, desembarque maior do que o embarcado.
    #[error(transparent)]
    Refused(DomainManifestError),

    /// O que é comum a toda a camada — validação, permissão, infraestrutura.
    #[error(transparent)]
    App(#[from] AppError),
}

impl From<DomainManifestError> for ManifestError {
    /// Separa campo recusado de estado recusado.
    ///
    /// "Quantidade tem que ser maior que zero" descreve o **campo** que chegou,
    /// não o estado do contêiner: é o mesmo tipo de recusa que um nome vazio, e
    /// sai como 422 junto com os outros. As demais variantes falam do pátio, e
    /// continuam sendo conflito.
    fn from(error: DomainManifestError) -> Self {
        match error {
            DomainManifestError::InvalidQuantity => {
                Self::App(AppError::Validation(vec![FieldError::new(
                    "quantity",
                    DomainManifestError::InvalidQuantity.to_string(),
                )]))
            }
            refused => Self::Refused(refused),
        }
    }
}

impl From<anyhow::Error> for ManifestError {
    fn from(error: anyhow::Error) -> Self {
        Self::App(AppError::Infra(error))
    }
}

impl From<crate::error::MetadataError> for ManifestError {
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
