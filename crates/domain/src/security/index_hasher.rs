//! O contrato do hash que vira chave de lookup.

/// Reduz um valor a uma chave curta e opaca de lookup.
///
/// O marcador guarda o digest, não o valor: a marca fica leve e o refresh token
/// original não pode ser reconstruído a partir do que está em memória.
pub trait IndexHasher {
    /// O digest do valor, estável entre chamadas e execuções.
    fn hash(&self, plain: &str) -> String;
}
