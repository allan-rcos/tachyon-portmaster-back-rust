//! O que `POST /products` recebe.

/// O que descreve o cadastro de um produto.
#[derive(Debug, Clone, Default)]
pub(crate) struct ProductCreateRequest {
    /// `name`.
    pub(crate) name: Option<String>,
    /// `density`.
    pub(crate) density: Option<f64>,
    /// `risk_class`.
    pub(crate) risk_class: Option<i32>,
}
