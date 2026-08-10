//! O que pode dar errado num caso de uso de usuário.

use portmaster_domain::error::{RoleError, UserError as DomainUserError};

use crate::error::AppError;

/// As falhas do serviço de usuários.
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    /// O usuário pedido não existe.
    #[error("usuário não encontrado: {0}")]
    Missing(String),

    /// Um dos papéis pedidos não existe.
    ///
    /// Separado de [`Self::Missing`] porque o id errado está num campo do corpo,
    /// e não na rota: quem lê a resposta precisa saber qual dos dois procurar.
    #[error("papel não encontrado: {0}")]
    MissingRole(String),

    /// Já existe um usuário com esse e-mail.
    ///
    /// Descoberto aqui em vez de deixar o índice único reclamar: assim o cliente
    /// recebe um conflito com sentido, e não um erro de banco.
    #[error("A user with this e-mail already exists.")]
    EmailTaken,

    /// O que é comum a toda a camada — validação, permissão, infraestrutura.
    #[error(transparent)]
    App(#[from] AppError),
}

impl From<DomainUserError> for UserError {
    fn from(error: DomainUserError) -> Self {
        match error {
            DomainUserError::Validation(fields) => Self::App(AppError::Validation(fields)),
        }
    }
}

impl From<RoleError> for UserError {
    /// Criar um usuário constrói os papéis dele, e o `TableModule` de papel pode
    /// recusar do mesmo jeito que o de usuário.
    fn from(error: RoleError) -> Self {
        match error {
            RoleError::Validation(fields) => Self::App(AppError::Validation(fields)),
        }
    }
}

impl From<anyhow::Error> for UserError {
    fn from(error: anyhow::Error) -> Self {
        Self::App(AppError::Infra(error))
    }
}

impl From<crate::error::MetadataError> for UserError {
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
