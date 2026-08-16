//! As impls de persistência sobre o `MariaDB`. Nenhuma sai do crate.

pub(crate) mod mariadb_repository_provider;

pub(crate) mod container_repository;
pub(crate) mod manifest_repository;
pub(crate) mod product_repository;
pub(crate) mod role_repository;
pub(crate) mod user_repository;

pub(crate) use mariadb_repository_provider::MariaDbRepositoryProvider;
