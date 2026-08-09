//! Os filtros de uma listagem paginada por cursor.

use serde::Deserialize;

/// Os filtros de uma listagem paginada por cursor.
///
/// Todos opcionais: uma listagem sem querystring é a primeira página com os
/// padrões da `infra`.
#[derive(Debug, Default, Deserialize)]
pub struct PageParams {
    /// Token da página anterior.
    pub(crate) cursor: Option<String>,
    /// Tamanho da página.
    pub(crate) limit: Option<u32>,
    /// Termo de busca.
    pub(crate) search: Option<String>,
}
