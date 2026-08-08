//! O que impede um usuário de existir.

use crate::error::FieldError;

/// Falhas ao construir ou alterar um usuário.
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    /// Um ou mais campos quebraram uma regra.
    #[error("dados de usuário inválidos")]
    Validation(Vec<FieldError>),
}
