//! O painel do pátio.

use crate::error::MetricsError;
use crate::queries::metrics::GetMetricsQuery;
use crate::services::MetadataUseCase;
use portmaster_infra::query::views::MetricsView;

/// O que a apresentação pode pedir sobre o painel.
#[trait_variant::make(Send)]
pub trait MetricsUseCase {
    /// Registra, no boot, as permissões que este serviço exige.
    ///
    /// Os slugs são `const` privadas da implementação e **não** saem dela: quem
    /// os compara com o `UserContext` é o próprio caso de uso, e não há segundo
    /// lugar no sistema que precise vê-los. O que atravessa esta fronteira é a
    /// ação de registrar, nunca a lista — é o molde do `declarePermission` do
    /// PHP, onde a permissão pertence a exatamente um caso de uso.
    async fn declare_permissions<M: MetadataUseCase + Send + Sync>(
        &self,
        registrar: &M,
    ) -> Result<(), MetricsError>;

    /// As oito agregações do pátio.
    async fn get(&self, query: GetMetricsQuery) -> Result<MetricsView, MetricsError>;
}
