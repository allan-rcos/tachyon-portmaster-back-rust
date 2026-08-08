//! Os parâmetros da listagem de usuários.

/// Os filtros da listagem de usuários.
///
/// Página e limite, não cursor: é a única consulta administrativa em que saltar
/// para uma página arbitrária é o uso real.
#[derive(Debug, Clone, Default)]
pub struct UserListParams {
    /// Página, começando em 1; ausente ou zero vale 1.
    pub page: Option<u32>,
    /// Tamanho da página; ausente usa [`DEFAULT_LIMIT`](crate::query::default_limit::DEFAULT_LIMIT).
    pub limit: Option<u32>,
}
