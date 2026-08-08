//! As regras de manifesto.

use crate::error::ManifestError;
use crate::models::{Container, ManifestCargo, ManifestChange, Product};

/// Movimenta carga, produzindo o efeito completo sobre contêiner e manifesto.
pub trait ManifestTM {
    /// Embarca uma quantidade de um produto.
    ///
    /// `current` é a linha de manifesto que já existe para esse produto, se
    /// houver — é o que permite somar ao que já está lá em vez de duplicar.
    fn load(
        &self,
        container: &dyn Container,
        product: &dyn Product,
        quantity: f64,
        current: Option<&dyn ManifestCargo>,
    ) -> Result<Box<dyn ManifestChange>, ManifestError>;

    /// Desembarca uma quantidade de um produto.
    fn unload(
        &self,
        container: &dyn Container,
        product: &dyn Product,
        quantity: f64,
        current: Option<&dyn ManifestCargo>,
    ) -> Result<Box<dyn ManifestChange>, ManifestError>;
}
