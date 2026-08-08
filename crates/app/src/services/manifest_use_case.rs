//! Carga e telemetria.

use crate::commands::manifest::MoveItemCommand;
use crate::error::AppError;
use portmaster_domain::models::Container;

/// O que a apresentação pode pedir sobre manifesto.
///
/// Os dois devolvem o **contêiner** no estado novo, e não a linha movimentada:
/// quem embarca quer saber quanto o contêiner passou a pesar e se saiu de vazio,
/// que é o que decide o próximo movimento. A linha em si o chamador já tem — foi
/// ele quem a pediu.
#[trait_variant::make(Send)]
pub trait ManifestUseCase {
    /// Embarca carga num contêiner, e devolve o contêiner resultante.
    async fn load(&self, command: MoveItemCommand) -> Result<Box<dyn Container>, AppError>;

    /// Desembarca carga de um contêiner, e devolve o contêiner resultante.
    async fn unload(&self, command: MoveItemCommand) -> Result<Box<dyn Container>, AppError>;
}
