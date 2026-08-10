//! O controller do painel. Não sai do módulo.

use portmaster_app::context::UserContext;
use portmaster_app::error::MetricsError;
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
            .map_err(to_api)?;

        Ok(MetricsXResponse::of(view))
    }
}

/// Traduz a recusa do serviço de métricas.
///
/// Ele só devolve o erro comum: o painel não endereça recurso nenhum.
fn to_api(error: MetricsError) -> ApiError {
    match error {
        MetricsError::App(shared) => ApiError::of_app(shared),
    }
}
