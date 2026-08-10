//! O que pode dar errado na leitura do painel.

use crate::error::AppError;

/// As falhas do serviço de métricas.
///
/// Só embrulha o comum: o painel não endereça recurso nenhum, então não tem
/// como pedir o que não existe nem contradizer estado.
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    /// O que é comum a toda a camada — permissão e infraestrutura.
    #[error(transparent)]
    App(#[from] AppError),
}

impl From<anyhow::Error> for MetricsError {
    fn from(error: anyhow::Error) -> Self {
        Self::App(AppError::Infra(error))
    }
}

impl From<crate::error::MetadataError> for MetricsError {
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
