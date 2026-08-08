//! `/metrics` — o painel do pátio.
//!
//! Uma rota só, sem parâmetro: os números são do pátio inteiro, e recortá-los
//! por filtro seria outra pergunta com outra resposta. O trabalho pesado é uma
//! consulta agregada na `infra`, atrás do cache de leitura.

use portmaster_app::queries::metrics::GetMetricsQuery;
use portmaster_app::services::MetricsUseCase;

use crate::error::api_error::ApiError;
use crate::session::Session;
use crate::wire::api_response::ApiResponse;
use crate::wire::dto::metrics::metrics_response_factory::MetricsResponseFactory;
use crate::wire::wire::Wire;

/// Os handlers de métrica.
pub struct MetricsHandlers<M> {
    metrics: M,
}

impl<M: MetricsUseCase> MetricsHandlers<M> {
    /// Monta os handlers.
    pub(crate) const fn new(metrics: M) -> Self {
        Self { metrics }
    }

    /// `GET /metrics`
    pub(crate) async fn get(&self, wire: Wire) -> Result<ApiResponse, ApiError> {
        let context = Session::require_user()?;

        let view = self
            .metrics
            .get(GetMetricsQuery { context })
            .await
            .map_err(ApiError::of_app)?;

        Ok(ApiResponse::ok(wire, MetricsResponseFactory::of(view)))
    }
}
