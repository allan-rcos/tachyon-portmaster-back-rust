//! O mapa das permissões registradas.

use std::sync::Arc;

use moka::future::Cache;

use crate::cache::interno::slug_cache::SlugCache;

/// O mapa das permissões.
///
/// É um tipo próprio, e não um `Arc<Cache<..>>` solto, porque os dois registros
/// **não podem** dividir o mesmo mapa: `PermissionRepository::all()` alimenta o
/// papel que o `POST /setup` cria, e um grupo de marcador caído ali vira uma
/// concessão que ninguém declarou. Tipos distintos transformam essa confusão num
/// erro de compilação em vez de numa permissão a mais no administrador.
#[derive(Clone)]
pub(crate) struct PermissionCache(pub(crate) SlugCache);

impl PermissionCache {
    /// Monta o mapa com a capacidade dada.
    pub(crate) fn new(capacity: u64) -> Self {
        Self(Arc::new(Cache::new(capacity)))
    }
}
