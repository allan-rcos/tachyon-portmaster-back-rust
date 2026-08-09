//! O controller do painel. Não sai do módulo.

use portmaster_app::context::UserContext;
use portmaster_app::queries::metrics::GetMetricsQuery;
use portmaster_app::services::MetricsUseCase;

use crate::controllers::metrics_controller::MetricsController;
use crate::error::api_error::ApiError;
use crate::wire::vo::metrics::metrics_x_response::MetricsXResponse;

/// Os handlers do painel, genéricos sobre o caso de uso.
#[derive(Clone)]
pub(crate) struct MetricsControllerImpl<M> {
    /// O caso de uso do painel.
    metrics: M,
}

impl<M: MetricsUseCase> MetricsControllerImpl<M> {
    /// Monta o controller.
    pub(crate) const fn new(metrics: M) -> Self {
        Self { metrics }
    }
}

impl<M: MetricsUseCase + Clone + Send + Sync + 'static> MetricsController
    for MetricsControllerImpl<M>
{
    async fn get(&self, context: UserContext) -> Result<MetricsXResponse, ApiError> {
        let view = self
            .metrics
            .get(GetMetricsQuery { context })
            .await
            .map_err(ApiError::of_app)?;

        Ok(MetricsXResponse::of(view))
    }
}
