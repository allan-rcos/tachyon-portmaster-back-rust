//! A conexão e a transação.

pub mod scope;
pub mod unit_of_work;

pub(crate) mod interno;
pub(crate) mod pool;

pub use scope::TransactionScope;
pub use unit_of_work::UnitOfWork;
