//! Registrar um grupo de marcador.

/// Declarar o grupo em que marcas poderão existir.
///
/// Roda no boot, uma vez. Sem `UserContext`: não há chamador a autorizar — o
/// registro acontece antes da primeira requisição, e é justamente o que faz o
/// repositório aceitar marcar naquele grupo depois.
#[derive(Debug, Clone)]
pub struct RegisterMarkerGroupCommand {
    /// O slug do grupo, em lower-kebab.
    pub slug: String,
}
