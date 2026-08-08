//! O mapa dos grupos de marcador registrados.

use std::sync::Arc;

use moka::future::Cache;

use crate::cache::interno::slug_cache::SlugCache;

/// O mapa dos grupos de marcador.
///
/// Tipo próprio pelo mesmo motivo do
/// [`PermissionCache`](super::permission_cache::PermissionCache): os dois
/// registros não podem dividir o mesmo mapa, e tipos distintos fazem disso um
/// erro de compilação.
#[derive(Clone)]
pub(crate) struct MarkerGroupCache(pub(crate) SlugCache);

impl MarkerGroupCache {
    /// Monta o mapa com a capacidade dada.
    pub(crate) fn new(capacity: u64) -> Self {
        Self(Arc::new(Cache::new(capacity)))
    }
}
