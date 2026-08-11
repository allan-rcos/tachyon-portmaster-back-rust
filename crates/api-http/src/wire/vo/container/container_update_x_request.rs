//! O VO de `ContainerUpdateRequest`.

use crate::ports::error::api_error::ApiError;
use crate::wire::dto::json::container::container_update_request_json::ContainerUpdateRequestJson;
use crate::wire::tables as fbs;
use crate::wire::x::request_x::RequestX;
use planus::ReadAsRoot as _;

/// O que a rota de `ContainerUpdateRequest` recebe.
///
/// Os campos são `Option` embora o `.fbs` marque alguns `required`: é o
/// que faz um campo ausente virar 422 nomeando-o, e não um 400 genérico.
#[derive(Debug, Clone, Default)]
pub(crate) struct ContainerUpdateXRequest {
    /// A capacidade máxima nova.
    pub(crate) max_capacity: Option<f64>,
}

impl RequestX for ContainerUpdateXRequest {
    type Json = ContainerUpdateRequestJson;

    fn of_json(dto: Self::Json) -> Self {
        Self {
            max_capacity: dto.max_capacity,
        }
    }

    /// Lê a mensagem do buffer, tolerando campo ausente.
    ///
    /// Campo declarado `required` no `.fbs` que não veio é buffer truncado —
    /// ilegível, não incompleto no sentido de negócio. O `ok()` deixa o `None`
    /// seguir para o `TableModule`, que dirá qual campo falta.
    fn of_fbs(bytes: &[u8]) -> Result<Self, ApiError> {
        let table = fbs::container::ContainerUpdateRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(Self {
            max_capacity: table.max_capacity().ok(),
        })
    }
}
