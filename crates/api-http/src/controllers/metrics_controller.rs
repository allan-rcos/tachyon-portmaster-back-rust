//! O contrato do controller do painel.

use crate::wire::api_response::ApiResponse;
use crate::wire::vo::metrics::metrics_x_response::MetricsXResponse;

/// Os handlers do painel do pátio.
#[trait_variant::make(Send)]
pub(crate) trait MetricsController: Clone + Sync + 'static {
    /// `GET /metrics`
    async fn get(self) -> ApiResponse<MetricsXResponse>;
}
