//! Digest de indexação com xxh64.
//!
//! Não é hash criptográfico e não deve ser usado como tal. O que se quer aqui é
//! uma chave curta, determinística e barata — um marcador é consultado a cada
//! refresh de token, e pagar Argon2 nisso seria absurdo.
//!
//! A saída são 16 caracteres hexadecimais, o que casa com a coluna `CHAR(16)`
//! que o modelo anterior usava.

use xxhash_rust::xxh64::xxh64;

use crate::security::IndexHasher;

/// Semente fixa. Precisa ser constante entre execuções: uma semente aleatória
/// por processo faria toda marca gravada antes de um restart virar inalcançável.
const SEED: u64 = 0;

/// Monta o digest de lookup para marcadores.
///
/// O que sai é o contrato: trocar o xxh64 por outro digest de indexação não
/// muda nenhuma assinatura fora deste arquivo — o que **não** pode mudar é a
/// [`SEED`], porque ela é o que liga o digest às marcas já gravadas.
pub(crate) const fn xx_index_hasher() -> impl IndexHasher + Send + Sync + Clone + use<> + 'static {
    XxIndexHasher
}

/// Digest de lookup para marcadores.
#[derive(Clone)]
struct XxIndexHasher;

impl IndexHasher for XxIndexHasher {
    fn hash(&self, plain: &str) -> String {
        format!("{:016x}", xxh64(plain.as_bytes(), SEED))
    }
}
