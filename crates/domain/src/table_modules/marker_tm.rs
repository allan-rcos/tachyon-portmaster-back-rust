//! As regras de marcador.

use crate::domain::Marker;
use crate::error::MarkerError;

/// Constrói marcadores, reduzindo o valor em claro a um digest.
pub trait MarkerTM {
    /// Cria um marcador para um valor, num grupo.
    fn create(
        &self,
        group: String,
        plain: &str,
        flag: bool,
    ) -> Result<Box<dyn Marker>, MarkerError>;
}
