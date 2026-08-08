//! O contrato de leitura de uma permissão.

/// Uma capacidade que um papel pode conceder.
///
/// O slug é tudo que uma permissão é. Não há rótulo nem descrição: o catálogo é
/// o próprio código — cada caso de uso declara a sua no boot — e um texto de
/// exibição seria uma segunda fonte de verdade a manter sincronizada com ele.
pub trait Permission: Send + Sync {
    /// O slug, em `domain:action`.
    fn slug(&self) -> &str;
}
