//! O filtro da listagem de permissões.

use serde::Deserialize;

/// O filtro da listagem de permissões.
#[derive(Debug, Default, Deserialize)]
pub struct SearchParams {
    /// Trecho do slug.
    pub(crate) search: Option<String>,
}
