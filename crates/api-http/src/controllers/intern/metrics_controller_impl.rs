//! O controller do painel. Não sai do módulo.

use portmaster_app::error::MetricsError;
use portmaster_app::queries::metrics::GetMetricsQuery;
use portmaster_app::services::MetricsService;

use crate::controllers::metrics_controller::MetricsController;
use crate::middleware::session_port::SessionPort;
use crate::ports::error::api_error::ApiError;
use crate::wire::api_response::ApiResponse;
use crate::wire::vo::metrics::metrics_x_response::MetricsXResponse;

/// Monta o controller de painel.
///
/// O service e o acesso à sessão chegam injetados, e o que sai é o contrato: o
/// tipo concreto não tem nome fora deste arquivo.
pub(crate) fn metrics_controller<M, S>(
    metrics: M,
    session: S,
) -> impl MetricsController + use<M, S> + 'static
where
    M: MetricsService + Clone + Send + Sync + 'static,
    S: SessionPort + Clone + Send + Sync + 'static,
{
    MetricsControllerImpl { metrics, session }
}

/// Os handlers do painel, genéricos sobre o service.
#[derive(Clone)]
struct MetricsControllerImpl<M, S> {
    /// O service do painel.
    metrics: M,
    /// Quem diz se há sessão, e quem a apresenta.
    session: S,
}

impl<M: MetricsService + Clone + Send + Sync + 'static, S: SessionPort> MetricsController
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
