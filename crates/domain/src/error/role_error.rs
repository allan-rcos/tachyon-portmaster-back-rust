//! O que impede um papel de existir.

use crate::error::FieldError;

/// Falhas ao construir ou alterar um papel.
#[derive(Debug, thiserror::Error)]
pub enum RoleError {
    /// Um ou mais campos quebraram uma regra.
    #[error("dados de papel inválidos")]
    Validation(Vec<FieldError>),
}
