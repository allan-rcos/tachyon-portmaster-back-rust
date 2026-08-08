//! O que impede um produto de existir.

use crate::error::FieldError;

/// Falhas ao construir ou alterar um produto.
#[derive(Debug, thiserror::Error)]
pub enum ProductError {
    /// Um ou mais campos quebraram uma regra.
    #[error("dados de produto inválidos")]
    Validation(Vec<FieldError>),
}
