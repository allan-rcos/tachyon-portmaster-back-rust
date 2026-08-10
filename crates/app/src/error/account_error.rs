//! O que pode dar errado num caso de uso da própria conta.

use portmaster_domain::error::{AuthError, UserError};

use crate::error::AppError;

/// As falhas do serviço de conta.
#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    /// A sessão não descreve mais ninguém, ou a senha atual não confere.
    ///
    /// Os dois casos são o mesmo erro de propósito. A conta some entre a emissão
    /// do token e o pedido quando o usuário é removido: o token continua
    /// assinado e válido, e não há mais dono para ele.
    #[error("Invalid e-mail or password.")]
    InvalidCredentials,

    /// O que é comum a toda a camada — validação, permissão, infraestrutura.
    #[error(transparent)]
    App(#[from] AppError),
}

impl From<AuthError> for AccountError {
    fn from(error: AuthError) -> Self {
        match error {
            AuthError::InvalidCredentials => Self::InvalidCredentials,
        }
    }
}

impl From<UserError> for AccountError {
    fn from(error: UserError) -> Self {
        match error {
            UserError::Validation(fields) => Self::App(AppError::Validation(fields)),
        }
    }
}

impl From<anyhow::Error> for AccountError {
    fn from(error: anyhow::Error) -> Self {
        Self::App(AppError::Infra(error))
    }
}
