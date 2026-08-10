//! O que pode dar errado num caso de uso de contêiner.

use portmaster_domain::error::ContainerError as DomainContainerError;

use crate::error::AppError;

/// As falhas do serviço de contêineres.
#[derive(Debug, thiserror::Error)]
pub enum ContainerError {
    /// O contêiner pedido não existe.
    #[error("contêiner não encontrado: {0}")]
    Missing(String),

    /// A operação contradiz o estado do pátio — selar o que não está
    /// carregando, despachar o que não está selado.
    ///
    /// Nada no pedido está errado; o contêiner é que não está no ponto de
    /// aceitá-lo.
    #[error(transparent)]
    Refused(DomainContainerError),

    /// O que é comum a toda a camada — validação, permissão, infraestrutura.
    #[error(transparent)]
    App(#[from] AppError),
}

impl From<DomainContainerError> for ContainerError {
    /// Separa campo recusado de estado recusado.
    ///
    /// `ContainerError` do domínio carrega os dois, e eles não são a mesma
    /// coisa: um código malformado é 422 como qualquer outro campo, e "esse
    /// contêiner não está selado" é conflito.
    fn from(error: DomainContainerError) -> Self {
        match error {
            DomainContainerError::Validation(fields) => Self::App(AppError::Validation(fields)),
            refused => Self::Refused(refused),
        }
    }
}

impl From<anyhow::Error> for ContainerError {
    fn from(error: anyhow::Error) -> Self {
        Self::App(AppError::Infra(error))
    }
}

impl From<crate::error::MetadataError> for ContainerError {
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
