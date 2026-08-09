//! O VO de `PUT /users/{id}/roles`.

use crate::error::api_error::ApiError;
use crate::wire::dto::json::admin::role_ids_request_json::RoleIdsRequestJson;
use crate::wire::x::request_x::RequestX;

/// O conjunto de papéis que um usuário passa a ter.
///
/// Não há tabela de `.fbs` para esta mensagem: ela nunca entrou no schema
/// publicado, e inventá-la agora mudaria o contrato de um endpoint em uso. Por
/// isso o caminho binário recusa — é a única rota do sistema que só aceita JSON,
/// e dizer isso explicitamente é melhor do que fingir um formato que o cliente
/// não tem como montar.
#[derive(Debug, Clone, Default)]
pub(crate) struct RoleIdsXRequest {
    /// O conjunto **final** de papéis; o que ficar de fora é retirado.
    pub(crate) role_ids: Option<Vec<String>>,
}

impl RequestX for RoleIdsXRequest {
    type Json = RoleIdsRequestJson;

    fn of_json(dto: Self::Json) -> Self {
        Self {
            role_ids: dto.role_ids,
        }
    }

    fn of_fbs(_bytes: &[u8]) -> Result<Self, ApiError> {
        Err(ApiError::unreadable_body(
            "esta rota só aceita application/json: a mensagem não está no schema FlatBuffers",
        ))
    }
}
