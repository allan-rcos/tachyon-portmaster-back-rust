//! O contexto de quem está fazendo a requisição.
//!
//! Chega por **argumento**, dentro de cada Command e Query — nunca de um estado
//! global. É o que permite a um caso de uso ser testado sem montar sessão, e o
//! que impede uma requisição de enxergar o contexto de outra.

pub mod role_context;
pub mod user_context;

pub use role_context::RoleContext;
pub use user_context::UserContext;
