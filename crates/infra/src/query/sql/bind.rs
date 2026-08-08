//! Um valor a ligar num placeholder.
//!
//! O tipo viaja junto porque o `sqlx` liga por tipo: um inteiro enviado como
//! string faz o `MariaDB` comparar número com texto e ignorar o índice.

/// Um valor a ligar no placeholder.
///
/// O tipo viaja junto porque o `sqlx` liga por tipo: um inteiro enviado como
/// string faz o `MariaDB` comparar número com texto e ignorar o índice.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Bind {
    /// Um inteiro — id, contagem, índice de enum.
    Int(i64),
    /// Um texto — termo de busca já normalizado.
    Text(String),
}
