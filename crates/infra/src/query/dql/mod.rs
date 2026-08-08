//! Os descritores de consulta: um por consulta.
//!
//! Um DQL não executa nada: ele descreve a consulta e sabe ler o resultado. Quem
//! executa é o `QueryRepository`, e é essa separação que faz uma consulta nova
//! não tocar o executor.
//!
//! Os `*_reader` são a exceção proposital: leitura de linha compartilhada entre
//! duas consultas que projetam o mesmo recorte.

pub(crate) mod account_reader;
pub(crate) mod container_reader;
pub(crate) mod get_account_dql;
pub(crate) mod get_container_dql;
pub(crate) mod get_product_dql;
pub(crate) mod get_role_dql;
pub(crate) mod list_container_summaries_dql;
pub(crate) mod list_containers_dql;
pub(crate) mod list_products_dql;
pub(crate) mod list_roles_dql;
pub(crate) mod list_users_dql;
pub(crate) mod metrics_dql;
pub(crate) mod paging;
pub(crate) mod product_reader;
pub(crate) mod role_reader;
