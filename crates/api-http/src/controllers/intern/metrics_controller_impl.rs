//! O controller do painel. Não sai do módulo.

use portmaster_app::error::MetricsError;
use portmaster_app::queries::metrics::GetMetricsQuery;
use portmaster_app::services::MetricsUseCase;

use crate::controllers::metrics_controller::MetricsController;
use crate::middleware::session_port::SessionPort;
use crate::ports::error::api_error::ApiError;
use crate::wire::api_response::ApiResponse;
use crate::wire::vo::metrics::metrics_x_response::MetricsXResponse;

/// Os handlers do painel, genéricos sobre o caso de uso.
#[derive(Clone)]
pub(crate) struct MetricsControllerImpl<M, S> {
    /// O caso de uso do painel.
    metrics: M,
    /// Quem diz se há sessão, e quem a apresenta.
    session: S,
}

impl<M: MetricsUseCase, S: SessionPort> MetricsControllerImpl<M, S> {
    /// Monta o controller.
    pub(crate) const fn new(metrics: M, session: S) -> Self {
        Self { metrics, session }
    }
}

impl<M: MetricsUseCase + Clone + Send + Sync + 'static, S: SessionPort> MetricsController
    for MetricsControllerImpl<M, S>
{
    async fn get(self) -> ApiResponse<MetricsXResponse> {
        ApiResponse::ok(
            async {
                let context = self.session.require_user()?;

                let view = self
                    .metrics
                    .get(GetMetricsQuery { context })
                    .await
                    .map_err(to_api)?;

                Ok(MetricsXResponse::of(view))
            }
            .await,
        )
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
