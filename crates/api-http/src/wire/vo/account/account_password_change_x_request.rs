//! O VO de `AccountPasswordChangeRequest`.

use crate::error::api_error::ApiError;
use crate::wire::dto::json::account::account_password_change_request_json::AccountPasswordChangeRequestJson;
use crate::wire::tables as fbs;
use crate::wire::x::request_x::RequestX;
use planus::ReadAsRoot as _;

/// O que a rota de `AccountPasswordChangeRequest` recebe.
///
/// Os campos são `Option` embora o `.fbs` marque alguns `required`: é o
/// que faz um campo ausente virar 422 nomeando-o, e não um 400 genérico.
#[derive(Debug, Clone, Default)]
pub(crate) struct AccountPasswordChangeXRequest {
    /// A senha atual, que prova ser o dono da conta.
    pub(crate) current_password: Option<String>,
    /// A senha nova.
    pub(crate) new_password: Option<String>,
}

impl RequestX for AccountPasswordChangeXRequest {
    type Json = AccountPasswordChangeRequestJson;

    fn of_json(dto: Self::Json) -> Self {
        Self {
            current_password: dto.current_password,
            new_password: dto.new_password,
        }
    }

    /// Lê a mensagem do buffer, tolerando campo ausente.
    ///
    /// Campo declarado `required` no `.fbs` que não veio é buffer truncado —
    /// ilegível, não incompleto no sentido de negócio. O `ok()` deixa o `None`
    /// seguir para o `TableModule`, que dirá qual campo falta.
    fn of_fbs(bytes: &[u8]) -> Result<Self, ApiError> {
        let table = fbs::account::AccountPasswordChangeRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(Self {
            current_password: table.current_password().ok().map(str::to_owned),
            new_password: table.new_password().ok().map(str::to_owned),
        })
    }
}
