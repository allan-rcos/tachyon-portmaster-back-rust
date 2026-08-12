//! Os factories dos services.

use portmaster_domain::id::{RandomIdGenerator, SequentialIdGenerator};
use portmaster_infra::logging::LoggerFactory;

use crate::services::{
    AccountService, ContainerService, ManifestService, MarkService, MetadataService,
    MetricsService, ProductService, RoleService, SessionService, UserService,
};

/// Os factories dos services.
///
/// Cada método devolve `impl Trait` — contrato, nunca tipo concreto. O tipo real
/// é innomeável: só existe depois da monomorfização, e é exatamente por isso que
/// a apresentação não consegue depender dele.
///
/// Os services são reconstruídos a cada chamada. Isso é barato de propósito:
/// eles não guardam estado, e os recursos caros — pool, caches — nasceram uma
/// vez no [`crate::register()`] e chegam por clone.
pub trait AppProvider {
    /// A conta do próprio usuário.
    fn account_service(&self) -> impl AccountService + Sync + Clone + use<Self> + 'static;

    /// Contêineres.
    fn container_service(&self) -> impl ContainerService + Sync + Clone + use<Self> + 'static;

    /// Carga e telemetria.
    fn manifest_service(&self) -> impl ManifestService + Sync + Clone + use<Self> + 'static;

    /// A primitiva de marcação — sessão de refresh é um uso dela.
    fn mark_service(&self) -> impl MarkService + Sync + Clone + use<Self> + 'static;

    /// Metadados de sistema.
    fn metadata_service(&self) -> impl MetadataService + Sync + Clone + use<Self> + 'static;

    /// O painel do pátio.
    fn metrics_service(&self) -> impl MetricsService + Sync + Clone + use<Self> + 'static;

    /// Produtos.
    fn product_service(&self) -> impl ProductService + Sync + Clone + use<Self> + 'static;

    /// Papéis.
    fn role_service(&self) -> impl RoleService + Sync + Clone + use<Self> + 'static;

    /// Login, validação de sessão e o setup inicial.
    fn session_service(&self) -> impl SessionService + Sync + Clone + use<Self> + 'static;

    /// Usuários.
    fn user_service(&self) -> impl UserService + Sync + Clone + use<Self> + 'static;

    /// Fábrica de loggers, para a apresentação.
    fn logger_factory(&self) -> impl LoggerFactory + use<Self> + 'static;

    /// Gerador de id opaco, para o refresh token.
    fn random_id_generator(&self) -> impl RandomIdGenerator + use<Self>;

    /// Gerador de id ordenável, para o `request_id`.
    fn sequential_id_generator(&self) -> impl SequentialIdGenerator + use<Self>;
}
