//! O mapa de slugs que os dois registros de metadado compartilham como *forma*.
//!
//! Compartilham a forma, nunca a instância: `PermissionCache` e
//! `MarkerGroupCache` são tipos distintos justamente para que o compilador
//! recuse trocá-los. Ver o doc de cada um.

use std::sync::Arc;

use moka::future::Cache;

/// Um conjunto de slugs registrados, vivo enquanto o processo estiver de pé.
pub(crate) type SlugCache = Arc<Cache<String, ()>>;
