//! O corpo de `PUT /users/{id}/roles`.

use serde::Deserialize;

/// O corpo de `PUT /users/{id}/roles`.
///
/// Não é uma tabela de `.fbs`: este payload nunca entrou no schema publicado, e
/// inventá-lo agora mudaria o contrato de um endpoint que já está em uso. Ver
/// [`JsonBody`](crate::wire::json_body::JsonBody).
#[derive(Debug, Default, Deserialize)]
pub struct RoleIdsRequest {
    /// O conjunto **final** de papéis; o que ficar de fora é retirado.
    #[serde(default)]
    pub(crate) role_ids: Vec<String>,
}
