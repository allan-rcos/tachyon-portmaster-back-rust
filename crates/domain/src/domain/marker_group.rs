//! O contrato de leitura de um grupo de marcador.

/// Um espaço de nomes de marcadores, registrado no boot.
///
/// Existe para que a `infra` possa recusar uma marca destinada a um grupo que
/// ninguém declarou — o que impede que um erro de digitação crie silenciosamente
/// um espaço de nomes paralelo em que nada é encontrado.
pub trait MarkerGroup: Send + Sync {
    /// O slug, em lower-kebab.
    fn slug(&self) -> &str;
}
