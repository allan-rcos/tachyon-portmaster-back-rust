//! Acesso ao MariaDB: o pool e a transação da requisição.

pub(crate) mod pool;
pub mod uow;

pub use uow::{in_scope, scope, UnitOfWork};
