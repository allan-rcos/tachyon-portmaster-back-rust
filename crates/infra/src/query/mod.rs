//! O lado de leitura.
//!
//! CQRS de verdade: a leitura não passa por repositório de agregado nem monta
//! objeto de domínio. Um DQL descreve **o que** consultar, o `QueryRepository`
//! sabe **executar**, e o resultado é uma `View` — struct POD, por valor,
//! monomorfizada. Nenhum `dyn` neste caminho.
//!
//! ## Como o `app` pede uma consulta
//!
//! Chamando a função dela em [`dql`], que devolve `impl SqlDql<View = …>`. Não
//! há factory: o `app` não recebe um objeto de onde escolher, recebe a função
//! que ele importou. Uma consulta nova é um arquivo novo, e nada mais.

pub mod default_limit;
pub mod dql;
pub mod dql_trait;
pub mod query_repository;
pub mod views;

pub(crate) mod column;
pub(crate) mod cursor;
pub(crate) mod intern;
pub(crate) mod query_provider;
pub(crate) mod sql_dql;

pub use default_limit::DEFAULT_LIMIT;
pub use dql_trait::Dql;
pub use query_repository::QueryRepository;

pub(crate) use query_provider::QueryProvider;
pub(crate) use sql_dql::SqlDql;
