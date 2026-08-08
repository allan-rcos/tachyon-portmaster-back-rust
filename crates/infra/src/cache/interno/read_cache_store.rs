//! O mapa por trás do cache de leitura.

use std::sync::Arc;

use moka::future::Cache;

/// Onde o cache de leitura guarda o que guarda.
///
/// Bytes e não a `View` tipada: uma `View` por tipo exigiria um cache por tipo,
/// e o que se quer é um só, agnóstico do que está passando por ele.
pub(crate) type ReadCacheStore = Arc<Cache<String, Arc<Vec<u8>>>>;
