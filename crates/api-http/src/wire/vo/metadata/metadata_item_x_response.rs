//! O VO de `MetadataItemResponse`.

use crate::wire::dto::json::metadata::metadata_item_response_json::MetadataItemResponseJson;
use crate::wire::tables as fbs;
use crate::wire::x::response_x::ResponseX;

/// O que a rota de `MetadataItemResponse` responde.
#[derive(Debug, Clone)]
pub(crate) struct MetadataItemXResponse {
    /// A identidade numérica do metadado.
    pub(crate) id: i32,
    /// O slug, que é como o resto do sistema o nomeia.
    pub(crate) slug: String,
}

impl ResponseX for MetadataItemXResponse {
    type Json = MetadataItemResponseJson;
    type Fbs = fbs::metadata::MetadataItemResponse;

    fn to_json(&self) -> Self::Json {
        MetadataItemResponseJson {
            id: self.id,
            slug: self.slug.clone(),
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::metadata::MetadataItemResponse {
            id: self.id,
            slug: Some(self.slug.clone()),
        }
    }
}
