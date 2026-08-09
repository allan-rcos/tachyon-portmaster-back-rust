//! O DTO de JSON de `MetadataItemResponse`.

use serde::Serialize;

/// `MetadataItemResponse` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct MetadataItemResponseJson {
    /// A identidade numérica do metadado.
    pub(crate) id: i32,
    /// O slug, que é como o resto do sistema o nomeia.
    pub(crate) slug: String,
}
