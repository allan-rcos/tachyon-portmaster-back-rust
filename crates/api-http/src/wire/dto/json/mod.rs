//! Os DTOs de JSON, gerados pelo `derive` do serde.
//!
//! São o par dos tipos que o planus gera do `.fbs`, e **não** os mesmos tipos:
//! o corpo textual tem forma própria, e uma mudança no schema binário não
//! deveria mexer nele sem que alguém peça.

pub(crate) mod account;
pub(crate) mod admin;
pub(crate) mod auth;
pub(crate) mod common;
pub(crate) mod container;
pub(crate) mod manifest;
pub(crate) mod metadata;
pub(crate) mod metrics;
pub(crate) mod product;
pub(crate) mod server;
