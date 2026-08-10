//! O que pode dar errado ao abrir ou validar uma sessão.

use portmaster_domain::error::{AuthError, RoleError, UserError};

use crate::error::AppError;

/// As falhas do serviço de sessão.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    /// Credencial recusada, ou sessão que não vale mais.
    ///
    /// E-mail desconhecido e senha errada produzem **o mesmo** erro de
    /// propósito: distinguir os dois entregaria a quem tentasse adivinhar uma
    /// lista de e-mails cadastrados.
    #[error("Invalid e-mail or password.")]
    InvalidCredentials,

    /// O sistema já tem usuário, e o `setup` só monta o primeiro.
    ///
    /// Sem isso, o endpoint seria uma porta aberta para criar um administrador a
    /// qualquer momento.
    #[error("This system has already been set up.")]
    AlreadySetUp,

    /// O que é comum a toda a camada — validação e infraestrutura.
    #[error(transparent)]
    App(#[from] AppError),
}

impl From<AuthError> for SessionError {
    fn from(error: AuthError) -> Self {
        match error {
            AuthError::InvalidCredentials => Self::InvalidCredentials,
        }
    }
}

impl From<UserError> for SessionError {
    fn from(error: UserError) -> Self {
        match error {
            UserError::Validation(fields) => Self::App(AppError::Validation(fields)),
        }
    }
}

impl From<RoleError> for SessionError {
    /// O `setup` cria o papel de administrador junto do primeiro usuário.
    fn from(error: RoleError) -> Self {
        match error {
            RoleError::Validation(fields) => Self::App(AppError::Validation(fields)),
        }
    }
}

impl From<anyhow::Error> for SessionError {
    fn from(error: anyhow::Error) -> Self {
        Self::App(AppError::Infra(error))
    }
}
