//! O VO de `RoleCreateRequest`.

use crate::error::api_error::ApiError;
use crate::wire::dto::json::admin::role_create_request_json::RoleCreateRequestJson;
use crate::wire::tables as fbs;
use crate::wire::x::request_x::RequestX;
use planus::ReadAsRoot as _;

/// O que a rota de `RoleCreateRequest` recebe.
///
/// Os campos são `Option` embora o `.fbs` marque alguns `required`: é o
/// que faz um campo ausente virar 422 nomeando-o, e não um 400 genérico.
#[derive(Debug, Clone, Default)]
pub(crate) struct RoleCreateXRequest {
    /// O nome do papel novo.
    pub(crate) name: Option<String>,
    /// Os slugs que ele concede.
    pub(crate) permissions: Option<Vec<String>>,
}

impl RequestX for RoleCreateXRequest {
    type Json = RoleCreateRequestJson;

    fn of_json(dto: Self::Json) -> Self {
        Self {
            name: dto.name,
            permissions: dto.permissions,
        }
    }

    /// Lê a mensagem do buffer, tolerando campo ausente.
    ///
    /// Campo declarado `required` no `.fbs` que não veio é buffer truncado —
    /// ilegível, não incompleto no sentido de negócio. O `ok()` deixa o `None`
    /// seguir para o `TableModule`, que dirá qual campo falta.
    fn of_fbs(bytes: &[u8]) -> Result<Self, ApiError> {
        let table = fbs::admin::RoleCreateRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(Self {
            name: table.name().ok().map(str::to_owned),
            permissions: table.permissions().ok().flatten().map(|items| {
                items
                    .into_iter()
                    .filter_map(|item| item.map(str::to_owned).ok())
                    .collect()
            }),
        })
    }
}
