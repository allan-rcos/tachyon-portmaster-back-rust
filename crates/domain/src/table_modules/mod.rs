//! Os `TableModules`: onde as regras de negócio moram.
//!
//! Um `TableModule` recebe **valores soltos** — nunca um `Command`, que é
//! vocabulário do `app` — valida-os, e devolve um objeto de domínio pronto ou o
//! erro tipado que explica por que não deu. É o único lugar do sistema que
//! constrói um model, e é por isso que os models não saem de `intern`.

pub mod auth_tm;
pub mod container_tm;
pub mod manifest_tm;
pub mod marker_group_tm;
pub mod marker_tm;
pub mod permission_tm;
pub mod product_tm;
pub mod role_tm;
pub mod user_tm;

pub(crate) mod intern;

pub use auth_tm::AuthTM;
pub use container_tm::ContainerTM;
pub use manifest_tm::ManifestTM;
pub use marker_group_tm::MarkerGroupTM;
pub use marker_tm::MarkerTM;
pub use permission_tm::PermissionTM;
pub use product_tm::ProductTM;
pub use role_tm::RoleTM;
pub use user_tm::UserTM;
