//! A falha que um caso de uso devolve — uma por serviço.
//!
//! Cada serviço tem o seu erro, e nele só as recusas que ele mesmo sabe
//! levantar: `ProductError::Missing` fala de produto, `ContainerError::Refused`
//! fala do pátio. O que é comum aos dez — o corpo malformado, a permissão que
//! falta, o banco que caiu — mora no [`AppError`] que todos embrulham.
//!
//! A divisão é para o controller. Quem atende `GET /products/{id}` precisa
//! mapear duas recusas, não onze, e as duas que lhe importam estão à vista na
//! assinatura do caso de uso que ele chama.

pub mod account_error;
pub mod app_error;
pub mod app_error_kind;
pub mod container_error;
pub mod manifest_error;
pub mod marker_error;
pub mod metadata_error;
pub mod metrics_error;
pub mod product_error;
pub mod role_error;
pub mod session_error;
pub mod user_error;

pub use account_error::AccountError;
pub use app_error::AppError;
pub use app_error_kind::AppErrorKind;
pub use container_error::ContainerError;
pub use manifest_error::ManifestError;
pub use marker_error::MarkerError;
pub use metadata_error::MetadataError;
pub use metrics_error::MetricsError;
pub use product_error::ProductError;
pub use role_error::RoleError;
pub use session_error::SessionError;
pub use user_error::UserError;
