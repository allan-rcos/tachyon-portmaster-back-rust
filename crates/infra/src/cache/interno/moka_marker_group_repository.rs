//! O repositório de grupos de marcador, em memória.

use portmaster_domain::models::MarkerGroup;

use crate::cache::interno::marker_group_cache::MarkerGroupCache;
use crate::repository::MarkerGroupRepository;

/// Registro de grupos de marcador.
pub struct MokaMarkerGroupRepository {
    /// Os grupos declarados no boot.
    cache: MarkerGroupCache,
}

impl MokaMarkerGroupRepository {
    /// Monta o registro sobre o mapa do processo.
    pub(crate) const fn new(cache: MarkerGroupCache) -> Self {
        Self { cache }
    }
}

impl MarkerGroupRepository for MokaMarkerGroupRepository {
    async fn register(&self, group: &dyn MarkerGroup) -> anyhow::Result<()> {
        self.cache.0.insert(group.slug().to_owned(), ()).await;
        Ok(())
    }

    async fn has(&self, slug: &str) -> anyhow::Result<bool> {
        Ok(self.cache.0.contains_key(slug))
    }
}
