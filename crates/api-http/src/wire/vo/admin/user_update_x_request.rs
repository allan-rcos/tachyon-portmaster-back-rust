//! O VO de `UserUpdateRequest`.

use crate::ports::error::api_error::ApiError;
use crate::wire::dto::json::admin::user_update_request_json::UserUpdateRequestJson;
use crate::wire::tables as fbs;
use crate::wire::x::request_x::RequestX;
use planus::ReadAsRoot as _;

/// O que a rota de `UserUpdateRequest` recebe.
///
/// Os campos são `Option` embora o `.fbs` marque alguns `required`: é o
/// que faz um campo ausente virar 422 nomeando-o, e não um 400 genérico.
#[derive(Debug, Clone, Default)]
pub(crate) struct UserUpdateXRequest {
    /// O nome novo.
    pub(crate) name: Option<String>,
    /// O e-mail novo.
    pub(crate) email: Option<String>,
}

impl RequestX for UserUpdateXRequest {
    type Json = UserUpdateRequestJson;

    fn of_json(dto: Self::Json) -> Self {
        Self {
            name: dto.name,
            email: dto.email,
        }
    }

    /// Lê a mensagem do buffer, tolerando campo ausente.
    ///
    /// Campo declarado `required` no `.fbs` que não veio é buffer truncado —
    /// ilegível, não incompleto no sentido de negócio. O `ok()` deixa o `None`
    /// seguir para o `TableModule`, que dirá qual campo falta.
    fn of_fbs(bytes: &[u8]) -> Result<Self, ApiError> {
        let table = fbs::admin::UserUpdateRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(Self {
            name: table.name().ok().map(str::to_owned),
            email: table.email().ok().map(str::to_owned),
        })
    }
}
