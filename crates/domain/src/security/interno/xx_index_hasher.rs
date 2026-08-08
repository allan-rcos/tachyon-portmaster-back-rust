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

/// Digest de lookup para marcadores.
pub struct XxIndexHasher;

impl XxIndexHasher {
    /// Monta o hasher.
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl IndexHasher for XxIndexHasher {
    fn hash(&self, plain: &str) -> String {
        format!("{:016x}", xxh64(plain.as_bytes(), SEED))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn o_mesmo_valor_da_sempre_o_mesmo_digest() {
        // O oposto do hasher de senha: aqui a estabilidade é o requisito, porque
        // é assim que se reencontra a marca gravada.
        let hasher = XxIndexHasher::new();
        assert_eq!(hasher.hash("refresh-abc"), hasher.hash("refresh-abc"));
    }

    #[test]
    fn valores_diferentes_dao_digests_diferentes() {
        let hasher = XxIndexHasher::new();
        assert_ne!(hasher.hash("refresh-abc"), hasher.hash("refresh-abd"));
    }

    #[test]
    fn o_digest_cabe_na_coluna_de_16() {
        let hasher = XxIndexHasher::new();
        for value in ["", "a", &"x".repeat(1000)] {
            let digest = hasher.hash(value);
            assert_eq!(
                digest.len(),
                16,
                "digest de {value:?} saiu com tamanho errado"
            );
            assert!(digest.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }
}
