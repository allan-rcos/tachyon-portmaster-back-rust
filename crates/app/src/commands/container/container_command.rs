//! Apontar um contêiner pelo id — apagar, selar, despachar.

use crate::context::UserContext;

/// Um comando que só identifica o contêiner.
///
/// Serve remover, selar e despachar: os três não têm dado próprio, e três
/// structs idênticas só criariam a chance de uma divergir das outras.
#[derive(Debug, Clone)]
pub struct ContainerCommand {
    /// Quem está agindo.
    pub context: UserContext,
    /// Id do contêiner, em base62.
    pub id: String,
}
