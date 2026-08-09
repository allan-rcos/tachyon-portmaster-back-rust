//! O VO de `UserAdminPasswordResetRequest`.

use crate::error::api_error::ApiError;
use crate::wire::dto::json::admin::user_admin_password_reset_request_json::UserAdminPasswordResetRequestJson;
use crate::wire::tables as fbs;
use crate::wire::x::request_x::RequestX;
use planus::ReadAsRoot as _;

/// O que a rota de `UserAdminPasswordResetRequest` recebe.
///
/// Os campos são `Option` embora o `.fbs` marque alguns `required`: é o
/// que faz um campo ausente virar 422 nomeando-o, e não um 400 genérico.
#[derive(Debug, Clone, Default)]
pub(crate) struct UserAdminPasswordResetXRequest {
    /// A senha nova, definida por quem administra.
    pub(crate) new_password: Option<String>,
}

impl RequestX for UserAdminPasswordResetXRequest {
    type Json = UserAdminPasswordResetRequestJson;

    fn of_json(dto: Self::Json) -> Self {
        Self {
            new_password: dto.new_password,
        }
    }

    /// Lê a mensagem do buffer, tolerando campo ausente.
    ///
    /// Campo declarado `required` no `.fbs` que não veio é buffer truncado —
    /// ilegível, não incompleto no sentido de negócio. O `ok()` deixa o `None`
    /// seguir para o `TableModule`, que dirá qual campo falta.
    fn of_fbs(bytes: &[u8]) -> Result<Self, ApiError> {
        let table = fbs::admin::UserAdminPasswordResetRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(Self {
            new_password: table.new_password().ok().map(str::to_owned),
        })
    }
}
