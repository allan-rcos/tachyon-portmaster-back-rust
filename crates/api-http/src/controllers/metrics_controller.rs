//! O contrato do controller do painel.

use portmaster_app::context::UserContext;

use crate::ports::error::api_error::ApiError;
use crate::wire::vo::metrics::metrics_x_response::MetricsXResponse;

/// Os handlers do painel do pátio.
#[trait_variant::make(Send)]
pub(crate) trait MetricsController: Clone + Sync + 'static {
    /// `GET /metrics`
    async fn get(&self, context: UserContext) -> Result<MetricsXResponse, ApiError>;
}
