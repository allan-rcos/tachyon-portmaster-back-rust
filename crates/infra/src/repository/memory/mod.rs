//! Os repositórios cujo armazenamento é a memória do processo.
//!
//! Escritos como os do `MariaDB`, e diferentes deles só no verbo: onde aqueles
//! têm um nome de tabela numa `const`, estes têm o **grupo**; onde aqueles
//! traduzem colunas, estes serializam. Para o `app` não há diferença nenhuma —
//! ele vê os mesmos ports, e não sabe qual dos dois está atendendo.

pub(crate) mod memory_repository_provider;

pub(crate) mod marker_group_repository;
pub(crate) mod marker_repository;
pub(crate) mod permission_repository;
pub(crate) mod view_cache_repository;

pub(crate) use memory_repository_provider::MemoryRepositoryProvider;
