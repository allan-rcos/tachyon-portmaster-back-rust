//! O VO de `PermissionListResponse`.

use crate::wire::dto::json::metadata::permission_list_response_json::PermissionListResponseJson;
use crate::wire::tables as fbs;
use crate::wire::vo::metadata::metadata_item_x_response::MetadataItemXResponse;
use crate::wire::x::response_x::ResponseX;

/// O que a rota de `PermissionListResponse` responde.
#[derive(Debug, Clone)]
pub(crate) struct PermissionListXResponse {
    /// As permissões que o sistema conhece.
    pub(crate) data: Vec<MetadataItemXResponse>,
}

impl ResponseX for PermissionListXResponse {
    type Json = PermissionListResponseJson;
    type Fbs = fbs::metadata::PermissionListResponse;

    fn to_json(&self) -> Self::Json {
        PermissionListResponseJson {
            data: self.data.iter().map(ResponseX::to_json).collect(),
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::metadata::PermissionListResponse {
            data: Some(self.data.iter().map(ResponseX::to_fbs).collect()),
        }
    }
}

impl PermissionListXResponse {
    /// A lista de permissões que o sistema conhece.
    ///
    /// O `id` é a posição na lista: o catálogo é uma constante compilada, não
    /// uma tabela, e o que identifica uma permissão é o slug.
    pub(crate) fn of(slugs: Vec<String>) -> Self {
        Self {
            data: slugs
                .into_iter()
                .enumerate()
                .map(|(index, slug)| MetadataItemXResponse {
                    id: i32::try_from(index).unwrap_or(i32::MAX),
                    slug,
                })
                .collect(),
        }
    }
}
