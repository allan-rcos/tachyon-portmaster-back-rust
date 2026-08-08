//! O painel do pátio.

use crate::error::AppError;
use crate::queries::metrics::GetMetricsQuery;
use portmaster_infra::query::views::MetricsView;

/// O que a apresentação pode pedir sobre o painel.
#[trait_variant::make(Send)]
pub trait MetricsUseCase {
    /// As oito agregações do pátio.
    async fn get(&self, query: GetMetricsQuery) -> Result<MetricsView, AppError>;
}
