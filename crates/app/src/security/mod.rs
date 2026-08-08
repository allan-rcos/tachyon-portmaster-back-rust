//! Autorização.
//!
//! O slug é contrato com o banco, o catálogo é o que o boot registra, e o
//! `RequiresPermission` é a checagem que abre todo caso de uso protegido —
//! antes do cache e antes da transação.

pub mod permission_catalog;
pub mod permission_slug;
pub mod refresh_token_group;

pub(crate) mod requires_permission;

pub use permission_catalog::PermissionCatalog;
pub use permission_slug::PermissionSlug;
pub use refresh_token_group::REFRESH_TOKEN_GROUP;
