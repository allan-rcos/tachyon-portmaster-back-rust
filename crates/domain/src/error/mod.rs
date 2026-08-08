//! Erros de domínio, um por arquivo.
//!
//! Regra de negócio devolve erro **tipado**, nunca `anyhow` e nunca um status
//! HTTP: o domínio não sabe que existe HTTP. Traduzir `SealRefused` em 409 é
//! trabalho do `api-http`, e concentrar isso lá é o que permite outra
//! apresentação — uma CLI, um daemon — reagir ao mesmo erro do jeito dela.
//!
//! Validação **acumula**. Um validator não para no primeiro campo inválido: ele
//! examina todos e devolve a lista inteira, para que o cliente conserte tudo de
//! uma vez em vez de descobrir um problema por requisição.

pub mod auth_error;
pub mod container_error;
pub mod field_error;
pub mod manifest_error;
pub mod marker_error;
pub mod metadata_error;
pub mod product_error;
pub mod role_error;
pub mod user_error;

pub(crate) mod interno;

pub use auth_error::AuthError;
pub use container_error::ContainerError;
pub use field_error::FieldError;
pub use manifest_error::ManifestError;
pub use marker_error::MarkerError;
pub use metadata_error::MetadataError;
pub use product_error::ProductError;
pub use role_error::RoleError;
pub use user_error::UserError;

pub(crate) use interno::validation::Validation;
