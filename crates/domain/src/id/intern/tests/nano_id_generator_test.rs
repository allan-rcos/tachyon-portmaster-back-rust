//! Os testes de `nano_id_generator`.

use super::*;
use std::collections::HashSet;

#[test]
fn o_token_aleatorio_nao_se_repete() {
    let generator = NanoIdGenerator::new();
    let ids: HashSet<String> = (0..1_000).map(|_| generator.next()).collect();

    assert_eq!(ids.len(), 1_000, "houve colisão em 1000 tokens");
}

#[test]
fn o_token_aleatorio_tem_a_entropia_esperada() {
    let generator = NanoIdGenerator::new();
    assert_eq!(generator.next().chars().count(), RANDOM_ID_SIZE);
}
