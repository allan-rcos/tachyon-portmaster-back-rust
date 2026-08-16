//! Quem serve os recortes da memória do processo.

use std::sync::OnceLock;

use crate::config::cache_limits::CacheLimits;
use crate::scope::memory::intern::moka_cache::{moka_cache, MokaCache};
use crate::scope::memory::intern::moka_store::moka_store;
use crate::scope::memory::memory_store::MemoryStore;

/// O mapa das permissões declaradas no boot.
static PERMISSIONS: OnceLock<MokaCache> = OnceLock::new();
/// O mapa dos grupos de marcador declarados no boot.
static MARKER_GROUPS: OnceLock<MokaCache> = OnceLock::new();
/// O mapa dos marcadores vivos — uma sessão de refresh cada.
static MARKERS: OnceLock<MokaCache> = OnceLock::new();
/// O mapa das consultas de leitura cacheadas.
static VIEWS: OnceLock<MokaCache> = OnceLock::new();

/// Os recortes da memória do processo, um por natureza de dado.
///
/// São quatro mapas e não um: o teto de cada um é diferente, e dividir o
/// armazenamento faria uma enxurrada de logins despejar o catálogo de
/// permissões — que é preenchido no boot e precisa valer enquanto o processo
/// valer.
///
/// Sem config: os tetos são `const` de build, porque quanto cabe em memória é
/// decisão de arquitetura e não de deploy. Os quatro mapas são guardados porque
/// **são** o estado — um cache remontado por chamada não é cache.
pub(crate) struct MemoryScopeProvider;

impl MemoryScopeProvider {
    /// O recorte das permissões.
    pub(crate) fn permissions() -> impl MemoryStore + Sync + Clone + use<> + 'static {
        moka_store(
            "permission",
            PERMISSIONS
                .get_or_init(|| moka_cache(CacheLimits::METADATA_CACHE_CAPACITY))
                .clone(),
        )
    }

    /// O recorte dos grupos de marcador.
    pub(crate) fn marker_groups() -> impl MemoryStore + Sync + Clone + use<> + 'static {
        moka_store(
            "marker-group",
            MARKER_GROUPS
                .get_or_init(|| moka_cache(CacheLimits::METADATA_CACHE_CAPACITY))
                .clone(),
        )
    }

    /// O recorte dos marcadores.
    pub(crate) fn markers() -> impl MemoryStore + Sync + Clone + use<> + 'static {
        moka_store(
            "marker",
            MARKERS
                .get_or_init(|| moka_cache(CacheLimits::MARKER_CACHE_CAPACITY))
                .clone(),
        )
    }

    /// O recorte do cache de leitura.
    pub(crate) fn views() -> impl MemoryStore + Sync + Clone + use<> + 'static {
        moka_store(
            "view",
            VIEWS
                .get_or_init(|| moka_cache(CacheLimits::READ_CACHE_CAPACITY))
                .clone(),
        )
    }
}
