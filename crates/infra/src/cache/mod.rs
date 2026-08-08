//! Os caches em memória.
//!
//! Três coisas diferentes moram aqui, e só a primeira é cache no sentido usual:
//!
//! * o **cache de leitura**, que absorve rajadas de consulta repetida;
//! * os **registros de metadado** (permissões, grupos de marcador), que são
//!   preenchidos no boot e nunca mais mudam — memória é o backing natural deles,
//!   não uma otimização;
//! * os **marcadores**, que são sessões de refresh vivas, com prazo próprio.

pub mod read_cache;

pub(crate) mod interno;

pub use read_cache::ReadCache;
