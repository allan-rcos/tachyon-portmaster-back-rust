//! O que impede carga de entrar ou sair de um contêiner.

/// Falhas ao embarcar ou desembarcar carga.
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    /// Quantidade nula ou negativa.
    #[error("Quantity must be greater than zero.")]
    InvalidQuantity,

    /// Contêiner selado ou já despachado não recebe carga.
    #[error("Cannot load a sealed or dispatched container.")]
    ContainerClosed,

    /// A carga não cabe.
    #[error("Loading this item would exceed the container capacity.")]
    ExceedsCapacity,

    /// Descarregar exige um contêiner em carregamento.
    #[error("Only a container in the loading state can be unloaded.")]
    UnloadRequiresLoading,

    /// Não há tanto daquele produto embarcado quanto se pediu para tirar.
    #[error("Not enough of this product is loaded to unload the requested quantity.")]
    InsufficientCargo,
}
