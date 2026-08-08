//! Embarcar ou desembarcar carga.

use crate::context::UserContext;

/// Movimentar carga — embarque ou desembarque.
///
/// Um comando só para os dois: o que muda é a operação pedida, não os dados.
#[derive(Debug, Clone)]
pub struct MoveItemCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Id do contêiner, em base62.
    pub container_id: String,
    /// Id do produto, em base62.
    pub product_id: String,
    /// Quantidade a movimentar.
    pub quantity: f64,
}
