//! O que `POST /containers/{id}/unload` recebe.

/// O que descreve um desembarque de carga.
#[derive(Debug, Clone, Default)]
pub(crate) struct UnloadItemRequest {
    /// `container_id`.
    pub(crate) container_id: Option<String>,
    /// `product_id`.
    pub(crate) product_id: Option<String>,
    /// `quantity`.
    pub(crate) quantity: Option<f64>,
}
