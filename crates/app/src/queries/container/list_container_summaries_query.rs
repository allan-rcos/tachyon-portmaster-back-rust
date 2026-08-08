//! Listar contêineres com carga e telemetria.

use crate::context::UserContext;

/// Listar contêineres com carga e telemetria recente.
#[derive(Debug, Clone)]
pub struct ListContainerSummariesQuery {
    /// Quem está consultando.
    pub context: UserContext,
    /// Restringe a um contêiner.
    pub id: Option<String>,
    /// Token da página anterior.
    pub cursor: Option<String>,
    /// Tamanho da página.
    pub limit: Option<u32>,
}
