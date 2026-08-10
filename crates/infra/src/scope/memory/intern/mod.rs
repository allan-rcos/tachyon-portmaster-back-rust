//! O contexto da memória e o handle que o publica. Não sai do crate.

pub(crate) mod moka_cache;
pub(crate) mod moka_context;
pub(crate) mod moka_layer;
pub(crate) mod moka_store;
pub(crate) mod pending_write;
