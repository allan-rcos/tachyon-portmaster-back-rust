//! Os filtros da listagem de usuários, que pagina por página e não por cursor.

use serde::Deserialize;

/// Os filtros da listagem de usuários, que pagina por página e não por cursor.
#[derive(Debug, Default, Deserialize)]
pub struct UserPageParams {
    /// Página, começando em 1.
    pub(crate) page: Option<u32>,
    /// Tamanho da página.
    pub(crate) limit: Option<u32>,
}
