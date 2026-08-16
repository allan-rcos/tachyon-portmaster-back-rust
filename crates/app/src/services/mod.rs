//! Os services: um trait por agregado.
//!
//! Um **service** é o agrupamento; cada método dele é um **caso de uso**. É a
//! mesma relação que um controller tem com os seus handlers, e é por isso que o
//! arquivo se chama `role_service.rs` e não `role_use_case.rs`: o que o arquivo
//! declara é o agrupamento, e `RoleService::create` é que é o caso de uso.
//!
//! O trait é o **port** de que a apresentação precisa — ela conhece só esta
//! camada. As implementações vivem em `intern` e não saem do crate: quem as
//! constrói é o `AppProvider`, e o que ele devolve é `impl Trait`.
//!
//! As permissões que cada serviço exige são `const` **privadas** da sua
//! implementação, em `intern/`. Elas não aparecem em trait nenhum e não saem do
//! arquivo: o que o trait expõe é `declare_permissions`, que o boot chama uma
//! vez por serviço — a ação de registrar atravessa a fronteira, o slug não.

pub mod account_service;
pub mod container_service;
pub mod manifest_service;
pub mod mark_service;
pub mod metadata_service;
pub mod metrics_service;
pub mod product_service;
pub mod role_service;
pub mod session_service;
pub mod user_service;

pub(crate) mod intern;
pub(crate) mod services_provider;

pub use account_service::AccountService;
pub use container_service::ContainerService;
pub use manifest_service::ManifestService;
pub use mark_service::MarkService;
pub use metadata_service::MetadataService;
pub use metrics_service::MetricsService;
pub use product_service::ProductService;
pub use role_service::RoleService;
pub use session_service::SessionService;
pub use user_service::UserService;

pub(crate) use services_provider::ServicesProvider;
