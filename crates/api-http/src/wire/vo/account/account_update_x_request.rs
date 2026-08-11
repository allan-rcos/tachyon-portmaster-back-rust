//! O VO de `AccountUpdateRequest`.

use crate::ports::error::api_error::ApiError;
use crate::wire::dto::json::account::account_update_request_json::AccountUpdateRequestJson;
use crate::wire::tables as fbs;
use crate::wire::x::request_x::RequestX;
use planus::ReadAsRoot as _;

/// O que a rota de `AccountUpdateRequest` recebe.
///
/// Os campos são `Option` embora o `.fbs` marque alguns `required`: é o
/// que faz um campo ausente virar 422 nomeando-o, e não um 400 genérico.
#[derive(Debug, Clone, Default)]
pub(crate) struct AccountUpdateXRequest {
    /// O nome novo.
    pub(crate) name: Option<String>,
    /// O e-mail novo.
    pub(crate) email: Option<String>,
}

impl RequestX for AccountUpdateXRequest {
    type Json = AccountUpdateRequestJson;

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
        let table = fbs::account::AccountUpdateRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(Self {
            name: table.name().ok().map(str::to_owned),
            email: table.email().ok().map(str::to_owned),
        })
    }
}
