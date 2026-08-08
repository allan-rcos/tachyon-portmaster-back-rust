//! Metadados de sistema.

use crate::error::AppError;
use crate::queries::metadata::ListPermissionsQuery;

/// O que a apresentação pode pedir sobre metadados.
#[trait_variant::make(Send)]
pub trait MetadataUseCase {
    /// Os slugs registrados, em ordem.
    async fn list_permissions(&self, query: ListPermissionsQuery) -> Result<Vec<String>, AppError>;
}
