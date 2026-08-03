//! `/metrics` — o painel do pátio.
//!
//! Uma rota só, sem parâmetro: os números são do pátio inteiro, e recortá-los
//! por filtro seria outra pergunta com outra resposta. O trabalho pesado é uma
//! consulta agregada na `infra`, atrás do cache de leitura.

use portmaster_app::metrics::{GetMetricsQuery, MetricsUseCase};

use crate::error::{app_error_to_status, ApiError};
use crate::session::Session;
use crate::wire::http::{Accept, Negotiated};
use crate::wire::tables as fbs;

/// Os handlers de métrica.
pub(crate) struct MetricsHandlers<M> {
    metrics: M,
}

impl<M: MetricsUseCase> MetricsHandlers<M> {
    /// Monta os handlers.
    pub(crate) fn new(metrics: M) -> Self {
        Self { metrics }
    }

    /// `GET /metrics`
    pub(crate) async fn get(
        &self,
        accept: Accept,
    ) -> Result<Negotiated<fbs::metrics::MetricsResponse>, ApiError> {
        let context = Session::require_user()?;

        let view = self
            .metrics
            .get(GetMetricsQuery { context })
            .await
            .map_err(app_error_to_status)?;

        Ok(Negotiated::ok(accept, view.into()))
    }
}
