//! O contrato de leitura de um marcador.

/// Um booleano marcado num grupo, sob um digest.
pub trait Marker: Send + Sync {
    /// Slug do grupo a que pertence.
    fn group(&self) -> &str;

    /// Digest do valor marcado — nunca o valor.
    fn key(&self) -> &str;

    /// O booleano em si.
    fn flag(&self) -> bool;
}
