//! Os descritores de consulta: um por consulta.
//!
//! Cada arquivo exporta **uma função**, e o descritor que ela devolve é privado
//! ao arquivo. O `app` chama a função e recebe um `impl SqlDql<View = …>`:
//! executável, mas opaco. Não há factory no meio — ela era um objeto com dez
//! métodos que só repassavam construtores, e cada consulta nova a fazia crescer.
//!
//! Onde duas consultas projetam o mesmo recorte, a leitura da linha mora no
//! arquivo da listagem e a consulta de item a chama. Duplicá-la seria a chance
//! de as duas divergirem na primeira coluna acrescentada.

pub(crate) mod get_account;
pub(crate) mod get_container;
pub(crate) mod get_product;
pub(crate) mod get_role;
pub(crate) mod list_container_summaries;
pub(crate) mod list_containers;
pub(crate) mod list_products;
pub(crate) mod list_roles;
pub(crate) mod list_users;
pub(crate) mod metrics;
pub(crate) mod paging;

pub use get_account::get_account;
pub use get_container::get_container;
pub use get_product::get_product;
pub use get_role::get_role;
pub use list_container_summaries::list_container_summaries;
pub use list_containers::list_containers;
pub use list_products::list_products;
pub use list_roles::list_roles;
pub use list_users::list_users;
pub use metrics::metrics;
