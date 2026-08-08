//! O que `POST /containers` recebe.

/// O que descreve o registro de um contêiner.
#[derive(Debug, Clone, Default)]
pub(crate) struct ContainerCreateRequest {
    /// `code`.
    pub(crate) code: Option<String>,
    /// `max_capacity`.
    pub(crate) max_capacity: Option<f64>,
}
