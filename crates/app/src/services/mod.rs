//! Os casos de uso: um trait por agregado.
//!
//! O trait é o **port** de que a apresentação precisa — ela conhece só esta
//! camada. As implementações vivem em `intern` e não saem do crate: quem as
//! constrói é o `AppProvider`, e o que ele devolve é `impl Trait`.
//!
//! As permissões que cada serviço exige são `const` **privadas** da sua
//! implementação, em `intern/`. Elas não aparecem em trait nenhum e não saem do
//! arquivo: o que o trait expõe é `declare_permissions`, que o boot chama uma
//! vez por serviço — a ação de registrar atravessa a fronteira, o slug não.

pub mod account_use_case;
pub mod container_use_case;
pub mod manifest_use_case;
pub mod mark_use_case;
pub mod metadata_use_case;
pub mod metrics_use_case;
pub mod product_use_case;
pub mod role_use_case;
pub mod session_use_case;
pub mod user_use_case;

pub(crate) mod intern;

pub use account_use_case::AccountUseCase;
pub use container_use_case::ContainerUseCase;
pub use manifest_use_case::ManifestUseCase;
pub use mark_use_case::MarkUseCase;
pub use metadata_use_case::MetadataUseCase;
pub use metrics_use_case::MetricsUseCase;
pub use product_use_case::ProductUseCase;
pub use role_use_case::RoleUseCase;
pub use session_use_case::SessionUseCase;
pub use user_use_case::UserUseCase;
