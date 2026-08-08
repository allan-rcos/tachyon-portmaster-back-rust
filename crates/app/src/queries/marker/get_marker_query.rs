//! Ler um marcador.

/// Consultar uma marca.
#[derive(Debug, Clone)]
pub struct GetMarkerQuery {
    /// O grupo.
    pub group: String,
    /// O valor em claro.
    pub value: String,
}
