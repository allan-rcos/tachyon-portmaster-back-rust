//! Gravar um marcador.

/// Marcar um valor.
#[derive(Debug, Clone)]
pub struct SetMarkerCommand {
    /// O grupo, que precisa estar registrado.
    pub group: String,
    /// O valor em claro — reduzido a digest antes de ser guardado.
    pub value: String,
    /// Ligar ou desligar a marca.
    pub flag: bool,
    /// Por quanto tempo a marca vale.
    pub ttl_seconds: u64,
}
