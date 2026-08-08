//! A falha ao ler uma string base62.

/// Falha ao decodificar uma string base62.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Base62Error {
    /// String vazia não representa número nenhum.
    #[error("base62 não decodifica string vazia")]
    Empty,

    /// Caractere fora do alfabeto.
    #[error("caractere base62 inválido: {0:?}")]
    InvalidCharacter(char),

    /// Valor grande demais para caber num id.
    ///
    /// Nenhum id emitido por esta aplicação chega aqui — um Snowflake estourar
    /// `i64` é problema de 2093. Um valor deste tamanho veio da URL, e é tão
    /// inválido quanto um caractere fora do alfabeto.
    #[error("valor base62 fora da faixa: {0}")]
    OutOfRange(String),
}
