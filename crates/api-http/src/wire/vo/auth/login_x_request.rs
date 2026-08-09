//! O VO de `LoginRequest`.

use crate::error::api_error::ApiError;
use crate::wire::dto::json::auth::login_request_json::LoginRequestJson;
use crate::wire::tables as fbs;
use crate::wire::x::request_x::RequestX;
use planus::ReadAsRoot as _;

/// O que a rota de `LoginRequest` recebe.
///
/// Os campos são `Option` embora o `.fbs` marque alguns `required`: é o
/// que faz um campo ausente virar 422 nomeando-o, e não um 400 genérico.
#[derive(Debug, Clone, Default)]
pub(crate) struct LoginXRequest {
    /// O e-mail informado.
    pub(crate) email: Option<String>,
    /// A senha em claro. Morre no `TableModule`, que guarda só o hash.
    pub(crate) password: Option<String>,
}

impl RequestX for LoginXRequest {
    type Json = LoginRequestJson;

    fn of_json(dto: Self::Json) -> Self {
        Self {
            email: dto.email,
            password: dto.password,
        }
    }

    /// Lê a mensagem do buffer, tolerando campo ausente.
    ///
    /// Campo declarado `required` no `.fbs` que não veio é buffer truncado —
    /// ilegível, não incompleto no sentido de negócio. O `ok()` deixa o `None`
    /// seguir para o `TableModule`, que dirá qual campo falta.
    fn of_fbs(bytes: &[u8]) -> Result<Self, ApiError> {
        let table = fbs::auth::LoginRequestRef::read_as_root(bytes)
            .map_err(|e| ApiError::unreadable_body(format!("corpo FlatBuffers inválido: {e}")))?;

        Ok(Self {
            email: table.email().ok().map(str::to_owned),
            password: table.password().ok().map(str::to_owned),
        })
    }
}
