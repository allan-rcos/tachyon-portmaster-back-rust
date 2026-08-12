//! Os testes de `argon2_hasher`.

use super::*;

#[test]
fn aceita_a_senha_correta_e_recusa_a_errada() {
    let hasher = Argon2Hasher::new();
    let hash = hasher.hash("Portmaster1");

    assert!(hasher.verify("Portmaster1", &hash));
    assert!(!hasher.verify("portmaster1", &hash));
    assert!(!hasher.verify("", &hash));
}

/// O salt precisa estar sendo aplicado.
///
/// Se estes coincidissem, o banco inteiro viraria alvo de uma tabela
/// pré-computada só.
#[test]
fn a_mesma_senha_gera_hashes_diferentes() {
    let hasher = Argon2Hasher::new();
    assert_ne!(hasher.hash("Portmaster1"), hasher.hash("Portmaster1"));
}

#[test]
fn hash_ilegivel_nao_autentica_ninguem() {
    let hasher = Argon2Hasher::new();
    assert!(!hasher.verify("qualquer", "não é um hash"));
    assert!(!hasher.verify("qualquer", ""));
}
