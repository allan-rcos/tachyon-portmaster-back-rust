//! Os factories dos casos de uso.

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
    fn account_use_case(&self) -> impl AccountUseCase + Sync;

    /// Contêineres.
    fn container_use_case(&self) -> impl ContainerUseCase + Sync;

    /// Carga e telemetria.
    fn manifest_use_case(&self) -> impl ManifestUseCase + Sync;

    /// A primitiva de marcação — sessão de refresh é um uso dela.
    fn mark_use_case(&self) -> impl MarkUseCase + Sync;

    /// Metadados de sistema.
    fn metadata_use_case(&self) -> impl MetadataUseCase + Sync;

    /// O painel do pátio.
    fn metrics_use_case(&self) -> impl MetricsUseCase + Sync;

    /// Produtos.
    fn product_use_case(&self) -> impl ProductUseCase + Sync;

    /// Papéis.
    fn role_use_case(&self) -> impl RoleUseCase + Sync;

    /// Login, validação de sessão e o setup inicial.
    fn session_use_case(&self) -> impl SessionUseCase + Sync;

    /// Usuários.
    fn user_use_case(&self) -> impl UserUseCase + Sync;

    /// Fábrica de loggers, para a apresentação.
    fn logger_factory(&self) -> impl LoggerFactory;

    /// Gerador de id opaco, para o refresh token.
    fn random_id_generator(&self) -> impl RandomIdGenerator;

    /// Gerador de id ordenável, para o `request_id`.
    fn sortable_id_generator(&self) -> impl SortableIdGenerator;
}
