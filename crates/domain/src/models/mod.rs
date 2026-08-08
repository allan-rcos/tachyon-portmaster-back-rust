//! Os objetos de domínio: um contrato de leitura por arquivo.
//!
//! Todo trait aqui é **somente-leitura** — só getters. Quem recebe um `User`
//! consegue lê-lo e não alterá-lo, e é isso que impede o `app` ou o `api` de
//! mudarem um e-mail sem passar pela validação do `TableModule`.
//!
//! As implementações vivem em `interno` e não saem do crate: construí-las é
//! privilégio do `TableModule` correspondente, que é quem conhece as regras.

pub mod container;
pub mod manifest_cargo;
pub mod manifest_change;
pub mod marker;
pub mod marker_group;
pub mod permission;
pub mod product;
pub mod role;
pub mod user;

pub(crate) mod interno;

pub use container::Container;
pub use manifest_cargo::ManifestCargo;
pub use manifest_change::ManifestChange;
pub use marker::Marker;
pub use marker_group::MarkerGroup;
pub use permission::Permission;
pub use product::Product;
pub use role::Role;
pub use user::User;
