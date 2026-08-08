//! O grupo de marcador em que as sessões de refresh vivem.

/// O grupo de marcador da sessão de refresh.
///
/// Registrado no boot pelo mesmo motivo das permissões: o
/// [`MarkerRepository`](portmaster_infra::repository::MarkerRepository) recusa
/// marcar num grupo que não conhece, e é o `api-http` — não esta camada — quem
/// vai usá-lo.
pub const REFRESH_TOKEN_GROUP: &str = "refresh-token";
