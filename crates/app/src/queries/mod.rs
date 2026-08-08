//! Os DTOs de leitura: um por consulta.
//!
//! Uma Query carrega o contexto de quem pergunta mais os filtros. O caso de uso
//! autoriza pelo contexto e repassa os filtros aos `Params` da `infra`.

pub mod account;
pub mod container;
pub mod marker;
pub mod metadata;
pub mod metrics;
pub mod product;
pub mod role;
pub mod user;
