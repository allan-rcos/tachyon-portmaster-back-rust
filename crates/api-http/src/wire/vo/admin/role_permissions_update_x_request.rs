//! O VO de `RolePermissionsUpdateRequest`.

use crate::ports::error::api_error::ApiError;
use crate::wire::dto::json::admin::role_permissions_update_request_json::RolePermissionsUpdateRequestJson;
use crate::wire::tables as fbs;
use crate::wire::x::request_x::RequestX;
use planus::ReadAsRoot as _;

/// O que a rota de `RolePermissionsUpdateRequest` recebe.
///
/// Os campos são `Option` embora o `.fbs` marque alguns `required`: é o
/// que faz um campo ausente virar 422 nomeando-o, e não um 400 genérico.
#[derive(Debug, Clone, Default)]
pub(crate) struct RolePermissionsUpdateXRequest {
    /// O conjunto **final** de permissões; o que ficar de fora é retirado.
    pub(crate) permissions: Option<Vec<String>>,
}

impl RequestX for RolePermissionsUpdateXRequest {
    type Json = RolePermissionsUpdateRequestJson;

    fn of_json(dto: Self::Json) -> Self {
        Self {
            permissions: dto.permissions,
        }
    }

    /// Lê a mensagem do buffer, tolerando campo ausente.
    ///
    /// Campo declarado `required` no `.fbs` que não veio é buffer truncado —
    /// ilegível, não incompleto no sentido de negócio. O `ok()` deixa o `None`
    /// seguir para o `TableModule`, que dirá qual campo falta.
    fn of_fbs(bytes: &[u8]) -> Result<Self, ApiError> {
        let table = fbs::admin::RolePermissionsUpdateRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(Self {
            permissions: table.permissions().ok().flatten().map(|items| {
                items
                    .into_iter()
                    .filter_map(|item| item.map(str::to_owned).ok())
                    .collect()
            }),
        })
    }
}
