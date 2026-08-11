//! O contrato do controller de estado do serviço.

use crate::ports::error::api_error::ApiError;
use crate::wire::vo::server::project_info_x::ProjectInfoX;

/// O handler que descreve o processo.
///
/// A única rota do sistema que não exige sessão nem toca o `app`: ela responde
/// sobre o próprio processo.
#[trait_variant::make(Send)]
pub(crate) trait ServerController: Clone + Sync + 'static {
    /// `GET /info`
    async fn info(&self) -> Result<ProjectInfoX, ApiError>;
}
