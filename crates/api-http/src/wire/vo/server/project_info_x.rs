//! O VO de `ProjectInfo`.

use crate::wire::dto::json::server::project_info_json::ProjectInfoJson;
use crate::wire::tables as fbs;
use crate::wire::x::response_x::ResponseX;

/// O que a rota de `ProjectInfo` responde.
#[derive(Debug, Clone)]
pub(crate) struct ProjectInfoX {
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

impl ResponseX for ProjectInfoX {
    type Json = ProjectInfoJson;
    type Fbs = fbs::server::ProjectInfo;

    fn to_json(&self) -> Self::Json {
        ProjectInfoJson {
            name: self.name.clone(),
            version: self.version.clone(),
            environment: self.environment.clone(),
            runtime: self.runtime.clone(),
            memory_usage_mb: self.memory_usage_mb,
        }
    }

    fn to_fbs(&self) -> Self::Fbs {
        fbs::server::ProjectInfo {
            name: Some(self.name.clone()),
            version: Some(self.version.clone()),
            environment: Some(self.environment.clone()),
            runtime: Some(self.runtime.clone()),
            memory_usage_mb: self.memory_usage_mb,
        }
    }
}
