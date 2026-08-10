//! Como esta camada é montada.
//!
//! O contrato dos factories, a implementação que os atende e a função que abre
//! o pool e devolve a camada pronta. Os três só fazem sentido lidos juntos, e
//! nenhum é chamado depois do boot.

pub mod provider;
pub mod register;

pub(crate) mod infra_provider;
