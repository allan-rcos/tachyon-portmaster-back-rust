//! As regras de contêiner.

use crate::error::ContainerError;
use crate::models::Container;

/// Constrói contêineres e é dono de todas as suas transições de status.
pub trait ContainerTM {
    /// Cria um contêiner novo, vazio e sem peso.
    fn create(&self, code: String, max_capacity: f64)
        -> Result<Box<dyn Container>, ContainerError>;

    /// Produz o contêiner com outra capacidade.
    fn update(
        &self,
        container: &dyn Container,
        max_capacity: f64,
    ) -> Result<Box<dyn Container>, ContainerError>;

    /// Sela o contêiner, fechando-o para carga.
    fn seal(&self, container: &dyn Container) -> Result<Box<dyn Container>, ContainerError>;

    /// Despacha o contêiner selado.
    fn dispatch(&self, container: &dyn Container) -> Result<Box<dyn Container>, ContainerError>;
}
