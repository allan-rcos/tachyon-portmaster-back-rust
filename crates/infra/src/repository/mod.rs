//! Os contratos de persistência: um por agregado.
//!
//! Um repositório fala em **objeto de domínio**, nunca em linha: recebe e
//! devolve `Box<dyn User>`, e a tradução para coluna acontece na entity. É o que
//! permite ao `app` orquestrar sem saber que existe SQL.

pub mod container_repository;
pub mod manifest_repository;
pub mod marker_group_repository;
pub mod marker_repository;
pub mod permission_repository;
pub mod product_repository;
pub mod role_repository;
pub mod user_repository;

pub(crate) mod mariadb;

pub use container_repository::ContainerRepository;
pub use manifest_repository::ManifestRepository;
pub use marker_group_repository::MarkerGroupRepository;
pub use marker_repository::MarkerRepository;
pub use permission_repository::PermissionRepository;
pub use product_repository::ProductRepository;
pub use role_repository::RoleRepository;
pub use user_repository::UserRepository;
