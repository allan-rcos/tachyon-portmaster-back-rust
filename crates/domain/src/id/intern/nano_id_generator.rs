//! O id opaco, pelo `NanoID`.

use crate::id::RandomIdGenerator;

/// Comprimento do id aleatório.
///
/// 21 caracteres do alfabeto URL-safe dão cerca de 126 bits de entropia — longe
/// de qualquer força bruta viável, e ainda cabendo num cookie sem incomodar.
const RANDOM_ID_SIZE: usize = 21;

/// Gerador de id opaco, sobre `NanoID`.
#[derive(Clone, Copy)]
pub(crate) struct NanoIdGenerator;

impl NanoIdGenerator {
    /// Monta o gerador.
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl RandomIdGenerator for NanoIdGenerator {
    fn next(&self) -> String {
        nanoid::nanoid!(RANDOM_ID_SIZE)
    }
}

#[cfg(test)]
mod tests {
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
}
