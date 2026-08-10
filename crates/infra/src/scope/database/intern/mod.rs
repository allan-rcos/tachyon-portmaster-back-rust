//! O contexto do banco e o handle que o abre. Não sai do crate.

pub(crate) mod mariadb_context;
pub(crate) mod mariadb_layer;
pub(crate) mod mariadb_pool;
pub(crate) mod mariadb_unit_of_work;
