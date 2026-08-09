//! Os VOs de mensagem — o que os controllers conhecem.
//!
//! Um por mensagem, independente de formato. Quem sabe virar bytes é a
//! strategy, pelas traits em [`crate::wire::x`].

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
