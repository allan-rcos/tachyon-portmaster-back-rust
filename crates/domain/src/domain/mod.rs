//! Os objetos de domínio: um contrato de leitura por arquivo.
//!
//! Todo trait aqui é **somente-leitura** — só getters. Quem recebe um [`User`]
//! consegue lê-lo e não alterá-lo, e é isso que impede o `app` ou o `api` de
//! mudarem um e-mail sem passar pela validação do `TableModule`.
//!
//! Nenhum leva sufixo, e a ausência é a informação: `User` é o domínio, e
//! `UserModel` — a implementação — é outra coisa, que mora dentro do
//! [`TableModule`](crate::table_modules) que a constrói e não sai de lá.

pub mod container;
pub mod manifest_cargo;
pub mod manifest_change;
pub mod marker;
pub mod marker_group;
pub mod permission;
pub mod product;
pub mod role;
pub mod user;

pub use container::Container;
pub use manifest_cargo::ManifestCargo;
pub use manifest_change::ManifestChange;
pub use marker::Marker;
pub use marker_group::MarkerGroup;
pub use permission::Permission;
pub use product::Product;
pub use role::Role;
pub use user::User;
