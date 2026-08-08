//! O que `PUT /products/{id}` recebe.

/// O que descreve a alteração de um produto.
#[derive(Debug, Clone, Default)]
pub(crate) struct ProductUpdateRequest {
    /// `name`.
    pub(crate) name: Option<String>,
    /// `density`.
    pub(crate) density: Option<f64>,
    /// `risk_class`.
    pub(crate) risk_class: Option<i32>,
}
