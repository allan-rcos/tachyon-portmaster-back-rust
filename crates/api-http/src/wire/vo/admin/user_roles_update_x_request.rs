//! O VO de `UserRolesUpdateRequest`.

use crate::ports::error::api_error::ApiError;
use crate::wire::dto::json::admin::user_roles_update_request_json::UserRolesUpdateRequestJson;
use crate::wire::tables as fbs;
use crate::wire::x::request_x::RequestX;
use planus::ReadAsRoot as _;

/// O conjunto de papéis que um usuário passa a ter.
///
/// A mensagem tem tabela no schema publicado, como todas as outras: sem ela, o
/// VO teria de responder 400 a quem negociasse `application/x-flatbuffers` —
/// que é o que o cliente de produção fala.
#[derive(Debug, Clone, Default)]
pub(crate) struct UserRolesUpdateXRequest {
    /// O conjunto **final** de papéis; o que ficar de fora é retirado.
    pub(crate) role_ids: Option<Vec<String>>,
}

impl RequestX for UserRolesUpdateXRequest {
    type Json = UserRolesUpdateRequestJson;

    fn of_json(dto: Self::Json) -> Self {
        Self {
            role_ids: dto.role_ids,
        }
    }

    /// Lê a mensagem do buffer.
    ///
    /// Um buffer que não abre é 400, e **não** uma lista vazia: o caso de uso lê
    /// lista vazia como "retirar todos os papéis", então tratar corpo ilegível
    /// como ausência de papéis revogaria o acesso do usuário com um 200. Vetor
    /// ausente é `None`, que é diferente de vetor presente e vazio — este último
    /// é o pedido legítimo de revogar tudo.
    fn of_fbs(bytes: &[u8]) -> Result<Self, ApiError> {
        let table = fbs::admin::UserRolesUpdateRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(Self {
            role_ids: table.role_ids().ok().flatten().map(|items| {
                items
                    .into_iter()
                    .filter_map(|item| item.map(str::to_owned).ok())
                    .collect()
            }),
        })
    }
}
