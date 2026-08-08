//! O que impede um metadado de sistema de ser registrado.

use crate::error::FieldError;

/// Falhas ao registrar um metadado de sistema (permissão, grupo de marcador).
#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    /// Slug fora do formato exigido.
    #[error("metadado de sistema inválido")]
    Validation(Vec<FieldError>),
}
