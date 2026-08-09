//! O DTO de JSON de `ProjectInfo`.

use serde::Serialize;

/// `ProjectInfo` como o serde o escreve.
#[derive(Debug, Serialize)]
pub(crate) struct ProjectInfoJson {
    /// O nome do serviço.
    pub(crate) name: String,
    /// A versão publicada.
    pub(crate) version: String,
    /// Em que ambiente ele está rodando.
    pub(crate) environment: String,
    /// A versão do compilador que o produziu.
    pub(crate) runtime: String,
    /// Quanta memória residente ele está usando.
    pub(crate) memory_usage_mb: f64,
}
