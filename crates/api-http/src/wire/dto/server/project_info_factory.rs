//! O que `GET /info` responde.

use crate::error::api_error::ApiError;
use crate::wire::factory::response_factory::ResponseFactory;
use crate::wire::tables as fbs;

/// Monta a tabela de identificação do processo.
pub(crate) struct ProjectInfoFactory {
    /// Nome de exibição.
    name: &'static str,
    /// A versão do binário, do `CARGO_PKG_VERSION`.
    version: &'static str,
    /// O nome do ambiente, como o `/info` o publica.
    environment: String,
    /// O que está executando, com a versão do compilador.
    runtime: String,
    /// A memória residente do processo, em MiB.
    memory_usage_mb: f64,
}

impl ProjectInfoFactory {
    /// Monta a factory com o que o processo sabe sobre si.
    pub(crate) const fn of(
        name: &'static str,
        version: &'static str,
        environment: String,
        runtime: String,
        memory_usage_mb: f64,
    ) -> Self {
        Self {
            name,
            version,
            environment,
            runtime,
            memory_usage_mb,
        }
    }
}

impl ResponseFactory for ProjectInfoFactory {
    type Table = fbs::server::ProjectInfo;

    fn table(&self) -> Result<Self::Table, ApiError> {
        Ok(fbs::server::ProjectInfo {
            name: Some(self.name.to_owned()),
            version: Some(self.version.to_owned()),
            environment: Some(self.environment.clone()),
            runtime: Some(self.runtime.clone()),
            memory_usage_mb: self.memory_usage_mb,
        })
    }
}
