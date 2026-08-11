//! O VO de `UserCreateRequest`.

use crate::ports::error::api_error::ApiError;
use crate::wire::dto::json::admin::user_create_request_json::UserCreateRequestJson;
use crate::wire::tables as fbs;
use crate::wire::x::request_x::RequestX;
use planus::ReadAsRoot as _;

/// O que a rota de `UserCreateRequest` recebe.
///
/// Os campos são `Option` embora o `.fbs` marque alguns `required`: é o
/// que faz um campo ausente virar 422 nomeando-o, e não um 400 genérico.
#[derive(Debug, Clone, Default)]
pub(crate) struct UserCreateXRequest {
    /// O nome do usuário novo.
    pub(crate) name: Option<String>,
    /// O e-mail, que também é a credencial de login.
    pub(crate) email: Option<String>,
    /// A senha inicial. Morre no `TableModule`, que guarda só o hash.
    pub(crate) initial_password: Option<String>,
    /// Os papéis com que ele nasce.
    pub(crate) role_ids: Option<Vec<String>>,
}

impl RequestX for UserCreateXRequest {
    type Json = UserCreateRequestJson;

    fn of_json(dto: Self::Json) -> Self {
        Self {
            name: dto.name,
            email: dto.email,
            initial_password: dto.initial_password,
            role_ids: dto.role_ids,
        }
    }

    /// Lê a mensagem do buffer, tolerando campo ausente.
    ///
    /// Campo declarado `required` no `.fbs` que não veio é buffer truncado —
    /// ilegível, não incompleto no sentido de negócio. O `ok()` deixa o `None`
    /// seguir para o `TableModule`, que dirá qual campo falta.
    fn of_fbs(bytes: &[u8]) -> Result<Self, ApiError> {
        let table = fbs::admin::UserCreateRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(Self {
            name: table.name().ok().map(str::to_owned),
            email: table.email().ok().map(str::to_owned),
            initial_password: table.initial_password().ok().map(str::to_owned),
            role_ids: table.role_ids().ok().flatten().map(|items| {
                items
                    .into_iter()
                    .filter_map(|item| item.map(str::to_owned).ok())
                    .collect()
            }),
        })
    }
}
