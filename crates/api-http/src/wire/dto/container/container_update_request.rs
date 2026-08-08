//! O que `PUT /containers/{id}` recebe.

/// O que descreve a alteração de um contêiner.
#[derive(Debug, Clone, Default)]
pub(crate) struct ContainerUpdateRequest {
    /// `max_capacity`.
    pub(crate) max_capacity: Option<f64>,
}
