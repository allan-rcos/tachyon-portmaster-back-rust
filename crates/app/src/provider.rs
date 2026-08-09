//! Os factories dos casos de uso.

use portmaster_infra::clock::Clock;
use portmaster_infra::id::{RandomIdGenerator, SortableIdGenerator};
use portmaster_infra::logging::LoggerFactory;

use crate::services::{
    AccountUseCase, ContainerUseCase, ManifestUseCase, MarkUseCase, MetadataUseCase,
    MetricsUseCase, ProductUseCase, RoleUseCase, SessionUseCase, UserUseCase,
};

/// Os factories dos casos de uso.
///
/// Cada método devolve `impl Trait` — contrato, nunca tipo concreto. O tipo real
/// é innomeável: só existe depois da monomorfização, e é exatamente por isso que
/// a apresentação não consegue depender dele.
///
/// Os casos de uso são reconstruídos a cada chamada. Isso é barato de propósito:
/// eles não guardam estado, e os recursos caros — pool, caches — nasceram uma
/// vez no [`crate::register::register`] e chegam por clone.
pub trait AppProvider {
    /// A conta do próprio usuário.
    fn account_use_case(&self) -> impl AccountUseCase + Sync + Clone + use<Self> + 'static;

    /// Contêineres.
    fn container_use_case(&self) -> impl ContainerUseCase + Sync + Clone + use<Self> + 'static;

    /// Carga e telemetria.
    fn manifest_use_case(&self) -> impl ManifestUseCase + Sync + Clone + use<Self> + 'static;

    /// A primitiva de marcação — sessão de refresh é um uso dela.
    fn mark_use_case(&self) -> impl MarkUseCase + Sync + Clone + use<Self> + 'static;

    /// Metadados de sistema.
    fn metadata_use_case(&self) -> impl MetadataUseCase + Sync + Clone + use<Self> + 'static;

    /// O painel do pátio.
    fn metrics_use_case(&self) -> impl MetricsUseCase + Sync + Clone + use<Self> + 'static;

    /// Produtos.
    fn product_use_case(&self) -> impl ProductUseCase + Sync + Clone + use<Self> + 'static;

    /// Papéis.
    fn role_use_case(&self) -> impl RoleUseCase + Sync + Clone + use<Self> + 'static;

    /// Login, validação de sessão e o setup inicial.
    fn session_use_case(&self) -> impl SessionUseCase + Sync + Clone + use<Self> + 'static;

    /// Usuários.
    fn user_use_case(&self) -> impl UserUseCase + Sync + Clone + use<Self> + 'static;

    /// Fábrica de loggers, para a apresentação.
    fn logger_factory(&self) -> impl LoggerFactory + use<Self> + 'static;

    /// Gerador de id opaco, para o refresh token.
    fn random_id_generator(&self) -> impl RandomIdGenerator + use<Self> + 'static;

    /// Gerador de id ordenável, para o `request_id`.
    fn sortable_id_generator(&self) -> impl SortableIdGenerator + use<Self> + 'static;

    /// A hora corrente, em UTC.
    fn clock(&self) -> impl Clock + use<Self> + 'static;
}
