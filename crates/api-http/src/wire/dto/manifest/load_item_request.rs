//! O que `POST /containers/{id}/load` recebe.

/// O que descreve um embarque de carga.
#[derive(Debug, Clone, Default)]
pub(crate) struct LoadItemRequest {
    /// `container_id`.
    pub(crate) container_id: Option<String>,
    /// `product_id`.
    pub(crate) product_id: Option<String>,
    /// `quantity`.
    pub(crate) quantity: Option<f64>,
}
