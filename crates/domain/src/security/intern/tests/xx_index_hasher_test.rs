//! Os testes de `xx_index_hasher`.

use super::*;
use pretty_assertions::assert_eq;

/// O oposto do hasher de senha: aqui a estabilidade **é** o requisito.
///
/// É assim que se reencontra a marca gravada — um digest que variasse por
/// chamada faria toda consulta de marcador falhar.
#[test]
fn o_mesmo_valor_da_sempre_o_mesmo_digest() {
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
