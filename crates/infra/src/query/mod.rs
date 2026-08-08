//! O lado de leitura.
//!
//! CQRS de verdade: a leitura não passa por repositório de agregado nem monta
//! objeto de domínio. Um DQL descreve **o que** consultar, o `QueryRepository`
//! sabe **executar**, e o resultado é uma `View` — struct POD, por valor,
//! monomorfizada. Nenhum `dyn` neste caminho.

pub mod default_limit;
pub mod dql_trait;
pub mod params;
pub mod query_factory;
pub mod query_repository;
pub mod views;

pub(crate) mod cursor;
pub(crate) mod dql;
pub(crate) mod interno;
pub(crate) mod row;
pub(crate) mod sql;
pub(crate) mod sql_dql;

pub use default_limit::DEFAULT_LIMIT;
pub use dql_trait::Dql;
pub use query_factory::QueryFactory;
pub use query_repository::QueryRepository;
pub use sql::SqlQuery;

pub(crate) use sql_dql::SqlDql;
