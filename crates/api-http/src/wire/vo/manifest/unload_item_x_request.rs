//! O VO de `UnloadItemRequest`.

use crate::error::api_error::ApiError;
use crate::wire::dto::json::manifest::unload_item_request_json::UnloadItemRequestJson;
use crate::wire::tables as fbs;
use crate::wire::x::request_x::RequestX;
use planus::ReadAsRoot as _;

/// O que a rota de `UnloadItemRequest` recebe.
///
/// Os campos são `Option` embora o `.fbs` marque alguns `required`: é o
/// que faz um campo ausente virar 422 nomeando-o, e não um 400 genérico.
#[derive(Debug, Clone, Default)]
pub(crate) struct UnloadItemXRequest {
    /// De qual contêiner desembarcar.
    pub(crate) container_id: Option<String>,
    /// O que desembarcar.
    pub(crate) product_id: Option<String>,
    /// Quantas unidades.
    pub(crate) quantity: Option<f64>,
}

impl RequestX for UnloadItemXRequest {
    type Json = UnloadItemRequestJson;

    fn of_json(dto: Self::Json) -> Self {
        Self {
            container_id: dto.container_id,
            product_id: dto.product_id,
            quantity: dto.quantity,
        }
    }

    /// Lê a mensagem do buffer, tolerando campo ausente.
    ///
    /// Campo declarado `required` no `.fbs` que não veio é buffer truncado —
    /// ilegível, não incompleto no sentido de negócio. O `ok()` deixa o `None`
    /// seguir para o `TableModule`, que dirá qual campo falta.
    fn of_fbs(bytes: &[u8]) -> Result<Self, ApiError> {
        let table = fbs::manifest::UnloadItemRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(Self {
            container_id: table.container_id().ok().flatten().map(str::to_owned),
            product_id: table.product_id().ok().flatten().map(str::to_owned),
            quantity: table.quantity().ok(),
        })
    }
}
