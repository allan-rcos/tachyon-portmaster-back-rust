//! O que pode dar errado num caso de uso de papel.

use portmaster_domain::error::RoleError as DomainRoleError;

use crate::error::AppError;

/// As falhas do serviço de papéis.
#[derive(Debug, thiserror::Error)]
pub enum RoleError {
    /// O papel pedido não existe.
    #[error("papel não encontrado: {0}")]
    Missing(String),

    /// O que é comum a toda a camada — validação, permissão, infraestrutura.
    #[error(transparent)]
    App(#[from] AppError),
}

impl From<DomainRoleError> for RoleError {
    fn from(error: DomainRoleError) -> Self {
        match error {
            DomainRoleError::Validation(fields) => Self::App(AppError::Validation(fields)),
        }
    }
}

impl From<anyhow::Error> for RoleError {
    fn from(error: anyhow::Error) -> Self {
        Self::App(AppError::Infra(error))
    }
}

impl From<crate::error::MetadataError> for RoleError {
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
