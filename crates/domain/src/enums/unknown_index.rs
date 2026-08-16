//! A recusa de um índice que nenhum enum reconhece.

use thiserror::Error;

/// Um índice que não corresponde a variante nenhuma.
///
/// Um só para todos os enums, e não um por arquivo: a recusa é sempre a mesma
/// frase com dois buracos, e o que muda entre um enum e outro é só o nome que
/// aparece nela.
///
/// Existir como erro, e não como `Option`, é o que permite à leitura de coluna
/// das entities dispensar o `ok_or_else` que cada `from_row` escrevia à mão — a
/// mensagem passa a morar junto do enum que a justifica.
#[derive(Debug, Error)]
#[error("{value} não corresponde a variante nenhuma de {enum_name}")]
pub struct UnknownIndex {
    /// O índice que veio do banco ou do fio.
    value: i32,
    /// O enum que o recusou.
    enum_name: &'static str,
}

impl UnknownIndex {
    /// Monta a recusa de um índice para um enum.
    #[must_use]
    pub const fn new(value: i32, enum_name: &'static str) -> Self {
        Self { value, enum_name }
    }
}
