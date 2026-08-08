//! O que impede um marcador de ser criado.

use crate::error::FieldError;

/// Falhas ao criar um marcador.
#[derive(Debug, thiserror::Error)]
pub enum MarkerError {
    /// Grupo ou valor inválidos.
    #[error("marcador inválido")]
    Validation(Vec<FieldError>),
}
