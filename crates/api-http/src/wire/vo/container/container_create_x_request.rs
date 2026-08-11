//! O VO de `ContainerCreateRequest`.

use crate::ports::error::api_error::ApiError;
use crate::wire::dto::json::container::container_create_request_json::ContainerCreateRequestJson;
use crate::wire::tables as fbs;
use crate::wire::x::request_x::RequestX;
use planus::ReadAsRoot as _;

/// O que a rota de `ContainerCreateRequest` recebe.
///
/// Os campos são `Option` embora o `.fbs` marque alguns `required`: é o
/// que faz um campo ausente virar 422 nomeando-o, e não um 400 genérico.
#[derive(Debug, Clone, Default)]
pub(crate) struct ContainerCreateXRequest {
    /// O código do contêiner.
    pub(crate) code: Option<String>,
    /// A capacidade máxima, em quilos.
    pub(crate) max_capacity: Option<f64>,
}

impl RequestX for ContainerCreateXRequest {
    type Json = ContainerCreateRequestJson;

    fn of_json(dto: Self::Json) -> Self {
        Self {
            code: dto.code,
            max_capacity: dto.max_capacity,
        }
    }

    /// Lê a mensagem do buffer, tolerando campo ausente.
    ///
    /// Campo declarado `required` no `.fbs` que não veio é buffer truncado —
    /// ilegível, não incompleto no sentido de negócio. O `ok()` deixa o `None`
    /// seguir para o `TableModule`, que dirá qual campo falta.
    fn of_fbs(bytes: &[u8]) -> Result<Self, ApiError> {
        let table = fbs::container::ContainerCreateRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(Self {
            code: table.code().ok().flatten().map(str::to_owned),
            max_capacity: table.max_capacity().ok(),
        })
    }
}
